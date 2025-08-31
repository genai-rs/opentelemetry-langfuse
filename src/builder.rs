//! Builder pattern for creating OpenTelemetry tracers with Langfuse integration.
//!
//! This module provides a fluent API for configuring and creating tracers
//! that automatically integrate with Langfuse's observability platform.

use crate::{
    context::TracingContext,
    mapper::{AttributeMapper, GenAIAttributeMapper},
    processor::LangfuseSpanProcessor,
};
use opentelemetry::{
    global,
    trace::Tracer,
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    runtime::Runtime,
    trace::{self, RandomIdGenerator, Sampler, SdkTracerProvider},
    Resource,
};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

/// Errors that can occur when building a Langfuse tracer.
#[derive(Error, Debug)]
pub enum BuilderError {
    /// Error from OpenTelemetry.
    #[error("OpenTelemetry error: {0}")]
    OpenTelemetry(#[from] opentelemetry_sdk::trace::TraceError),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Missing required configuration.
    #[error("Missing required configuration: {0}")]
    MissingConfiguration(String),
}

/// Result type for builder operations.
pub type BuilderResult<T> = Result<T, BuilderError>;

/// Default Langfuse OTLP endpoint.
pub const DEFAULT_LANGFUSE_ENDPOINT: &str = "https://cloud.langfuse.com/api/public/otel";

/// Builder for creating a Langfuse-integrated OpenTelemetry tracer.
pub struct LangfuseTracerBuilder<R: Runtime> {
    endpoint: String,
    context: TracingContext,
    mapper: Arc<dyn AttributeMapper>,
    service_name: String,
    service_version: Option<String>,
    headers: Vec<(String, String)>,
    timeout: Duration,
    sampler: Sampler,
    runtime: R,
    batch_config: BatchConfig,
}

/// Configuration for batch processing.
#[derive(Clone)]
pub struct BatchConfig {
    /// Maximum queue size for pending spans.
    pub max_queue_size: usize,
    /// Maximum batch size for export.
    pub max_export_batch_size: usize,
    /// Delay between export attempts.
    pub scheduled_delay: Duration,
    /// Maximum timeout for export.
    pub max_export_timeout: Duration,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 2048,
            max_export_batch_size: 512,
            scheduled_delay: Duration::from_secs(5),
            max_export_timeout: Duration::from_secs(30),
        }
    }
}

impl<R: Runtime> LangfuseTracerBuilder<R> {
    /// Creates a new builder with the specified runtime.
    pub fn new(runtime: R) -> Self {
        Self {
            endpoint: DEFAULT_LANGFUSE_ENDPOINT.to_string(),
            context: TracingContext::new(),
            mapper: Arc::new(GenAIAttributeMapper::new()),
            service_name: "langfuse-otel".to_string(),
            service_version: None,
            headers: Vec::new(),
            timeout: Duration::from_secs(10),
            sampler: Sampler::AlwaysOn,
            runtime,
            batch_config: BatchConfig::default(),
        }
    }

    /// Sets the OTLP endpoint.
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }

    /// Sets the tracing context.
    pub fn with_context(mut self, context: TracingContext) -> Self {
        self.context = context;
        self
    }

    /// Sets a custom attribute mapper.
    pub fn with_mapper(mut self, mapper: Arc<dyn AttributeMapper>) -> Self {
        self.mapper = mapper;
        self
    }

    /// Sets the service name.
    pub fn with_service_name(mut self, name: impl Into<String>) -> Self {
        self.service_name = name.into();
        self
    }

    /// Sets the service version.
    pub fn with_service_version(mut self, version: impl Into<String>) -> Self {
        self.service_version = Some(version.into());
        self
    }

    /// Adds an HTTP header to be sent with OTLP requests.
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }

    /// Sets the Langfuse API key as a header.
    pub fn with_api_key(self, api_key: impl Into<String>) -> Self {
        self.with_header("x-langfuse-api-key", api_key)
    }

    /// Sets the export timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the sampling strategy.
    pub fn with_sampler(mut self, sampler: Sampler) -> Self {
        self.sampler = sampler;
        self
    }

    /// Configures batch processing.
    pub fn with_batch_config(mut self, config: BatchConfig) -> Self {
        self.batch_config = config;
        self
    }

    /// Sets the session ID in the context.
    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.context = self.context.with_session(session_id);
        self
    }

    /// Sets the user ID in the context.
    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        self.context = self.context.with_user(user_id);
        self
    }

    /// Adds metadata to the context.
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.context = self.context.with_metadata(key, value);
        self
    }

    /// Builds the tracer.
    pub fn build(self) -> BuilderResult<impl Tracer> {
        // Create resource attributes
        let mut resource_attrs = vec![KeyValue::new("service.name", self.service_name.clone())];

        if let Some(version) = self.service_version {
            resource_attrs.push(KeyValue::new("service.version", version));
        }

        // Add context attributes to resource
        resource_attrs.extend(self.context.to_otel_attributes());

        let resource = Resource::new(resource_attrs);

        // Use the existing exporter builder from our crate
        let mut exporter_builder = crate::exporter::ExporterBuilder::new()
            .with_endpoint(&self.endpoint)
            .with_timeout(self.timeout);

        // Add headers
        for (key, value) in self.headers {
            exporter_builder = exporter_builder.with_header(&key, &value);
        }

        // Build the exporter
        let span_exporter = exporter_builder
            .build()
            .map_err(|e| BuilderError::Configuration(e.to_string()))?;

        // Create the span processor with Langfuse context
        let processor = LangfuseSpanProcessor::builder(span_exporter, self.runtime.clone())
            .with_context(self.context)
            .with_mapper(self.mapper)
            .with_max_queue_size(self.batch_config.max_queue_size)
            .with_max_export_batch_size(self.batch_config.max_export_batch_size)
            .with_scheduled_delay(self.batch_config.scheduled_delay)
            .with_max_export_timeout(self.batch_config.max_export_timeout)
            .build();

        // Create the tracer provider
        let provider = SdkTracerProvider::builder()
            .with_span_processor(processor)
            .with_config(
                trace::Config::default()
                    .with_sampler(self.sampler)
                    .with_id_generator(RandomIdGenerator::default())
                    .with_resource(resource),
            )
            .build();

        // Get a tracer from the provider
        let tracer = provider.tracer(self.service_name);

        // Set as global provider if desired
        global::set_tracer_provider(provider);

        Ok(tracer)
    }
}

/// Simplified builder function for Tokio runtime.
#[cfg(feature = "tokio")]
pub fn builder() -> LangfuseTracerBuilder<opentelemetry_sdk::runtime::Tokio> {
    LangfuseTracerBuilder::new(opentelemetry_sdk::runtime::Tokio)
}

/// Quick setup function for common use cases.
pub async fn init_tracer(
    service_name: impl Into<String>,
    api_key: impl Into<String>,
) -> BuilderResult<impl Tracer> {
    #[cfg(feature = "tokio")]
    {
        builder()
            .with_service_name(service_name)
            .with_api_key(api_key)
            .build()
    }

    #[cfg(not(feature = "tokio"))]
    {
        Err(BuilderError::Configuration(
            "Tokio runtime feature is not enabled".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry_sdk::runtime::Tokio;
    use serde_json::json;

    #[test]
    fn test_builder_configuration() {
        let builder = LangfuseTracerBuilder::new(Tokio)
            .with_endpoint("https://example.com/otel")
            .with_service_name("test-service")
            .with_service_version("1.0.0")
            .with_api_key("test-key")
            .with_session("session-123")
            .with_user("user-456")
            .with_metadata("environment", json!("testing"))
            .with_timeout(Duration::from_secs(30));

        // Verify builder properties are set
        assert_eq!(builder.endpoint, "https://example.com/otel");
        assert_eq!(builder.service_name, "test-service");
        assert_eq!(builder.service_version, Some("1.0.0".to_string()));
        assert_eq!(builder.timeout, Duration::from_secs(30));

        // Check that API key header was added
        assert!(builder
            .headers
            .iter()
            .any(|(k, _)| k == "x-langfuse-api-key"));
    }

    #[test]
    fn test_batch_config() {
        let config = BatchConfig {
            max_queue_size: 4096,
            max_export_batch_size: 1024,
            scheduled_delay: Duration::from_secs(10),
            max_export_timeout: Duration::from_secs(60),
        };

        let builder = LangfuseTracerBuilder::new(Tokio).with_batch_config(config.clone());

        assert_eq!(builder.batch_config.max_queue_size, 4096);
        assert_eq!(builder.batch_config.max_export_batch_size, 1024);
    }

    #[test]
    fn test_default_values() {
        let builder = LangfuseTracerBuilder::new(Tokio);

        assert_eq!(builder.endpoint, DEFAULT_LANGFUSE_ENDPOINT);
        assert_eq!(builder.service_name, "langfuse-otel");
        assert!(builder.service_version.is_none());
        assert_eq!(builder.timeout, Duration::from_secs(10));
        assert!(matches!(builder.sampler, Sampler::AlwaysOn));
    }
}
