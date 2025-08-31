//! Tracer configuration for Langfuse integration.

use crate::{auth, endpoint, Error, Result};
use opentelemetry::global;
use opentelemetry_otlp::{WithExportConfig, WithHttpConfig};
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::Resource;
use std::collections::HashMap;

/// Builder for configuring a Langfuse tracer.
pub struct TracerBuilder {
    service_name: String,
    endpoint: Option<String>,
    auth_header: Option<String>,
    resource_attributes: Vec<opentelemetry::KeyValue>,
}

impl TracerBuilder {
    /// Creates a new TracerBuilder with the given service name.
    ///
    /// # Arguments
    ///
    /// * `service_name` - The name of the service for tracing
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            endpoint: None,
            auth_header: None,
            resource_attributes: Vec::new(),
        }
    }

    /// Sets the Langfuse endpoint URL.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The complete OTLP endpoint URL
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    /// Sets the authentication header.
    ///
    /// # Arguments
    ///
    /// * `auth_header` - The complete authentication header value
    pub fn with_auth_header(mut self, auth_header: impl Into<String>) -> Self {
        self.auth_header = Some(auth_header.into());
        self
    }

    /// Sets the Langfuse credentials directly.
    ///
    /// # Arguments
    ///
    /// * `public_key` - The Langfuse public key
    /// * `secret_key` - The Langfuse secret key
    pub fn with_credentials(mut self, public_key: &str, secret_key: &str) -> Self {
        self.auth_header = Some(auth::build_auth_header(public_key, secret_key));
        self
    }

    /// Sets the Langfuse host URL.
    ///
    /// # Arguments
    ///
    /// * `host` - The base Langfuse URL (e.g., `https://cloud.langfuse.com`)
    pub fn with_host(mut self, host: &str) -> Self {
        self.endpoint = Some(endpoint::build_otlp_endpoint(host));
        self
    }

    /// Adds a resource attribute.
    ///
    /// # Arguments
    ///
    /// * `key` - The attribute key
    /// * `value` - The attribute value
    pub fn with_resource_attribute<V>(mut self, key: &'static str, value: V) -> Self
    where
        V: Into<opentelemetry::Value>,
    {
        self.resource_attributes
            .push(opentelemetry::KeyValue::new(key, value));
        self
    }

    /// Loads configuration from environment variables.
    ///
    /// This method reads:
    /// - LANGFUSE_HOST for the endpoint
    /// - LANGFUSE_PUBLIC_KEY and LANGFUSE_SECRET_KEY for authentication
    pub fn from_env(mut self) -> Result<Self> {
        self.endpoint = Some(endpoint::build_otlp_endpoint_from_env()?);
        self.auth_header = Some(auth::build_auth_header_from_env()?);
        Ok(self)
    }

    /// Builds and installs the tracer as the global tracer provider.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the TracerProvider if successful.
    pub fn install(self) -> Result<TracerProvider> {
        let endpoint = self
            .endpoint
            .ok_or(Error::MissingEnvironmentVariable("endpoint"))?;
        let auth_header = self
            .auth_header
            .ok_or(Error::MissingEnvironmentVariable("auth_header"))?;

        // Create headers for authentication
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), auth_header);

        // Create OTLP exporter
        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_http()
            .with_endpoint(endpoint)
            .with_headers(headers)
            .build()?;

        // Add service.name to resource attributes
        let mut resource_attributes = self.resource_attributes;
        resource_attributes.push(opentelemetry::KeyValue::new(
            "service.name",
            self.service_name,
        ));

        // Create tracer provider
        let tracer_provider = TracerProvider::builder()
            .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
            .with_resource(Resource::new(resource_attributes))
            .build();

        // Set as global provider
        global::set_tracer_provider(tracer_provider.clone());

        Ok(tracer_provider)
    }
}

/// Initializes a tracer with Langfuse backend using environment variables.
///
/// This is a convenience function that reads configuration from environment variables:
/// - LANGFUSE_HOST: The base URL of your Langfuse instance
/// - LANGFUSE_PUBLIC_KEY: Your Langfuse public key
/// - LANGFUSE_SECRET_KEY: Your Langfuse secret key
///
/// # Arguments
///
/// * `service_name` - The name of the service for tracing
///
/// # Returns
///
/// Returns a Result containing the TracerProvider if successful.
///
/// # Example
///
/// ```no_run
/// use opentelemetry_langfuse::init_tracer_from_env;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let tracer_provider = init_tracer_from_env("my-service")?;
/// // Your application code here
/// # Ok(())
/// # }
/// ```
pub fn init_tracer_from_env(service_name: impl Into<String>) -> Result<TracerProvider> {
    TracerBuilder::new(service_name).from_env()?.install()
}

/// Initializes a tracer with Langfuse backend using explicit configuration.
///
/// # Arguments
///
/// * `service_name` - The name of the service for tracing
/// * `host` - The base Langfuse URL (e.g., `https://cloud.langfuse.com`)
/// * `public_key` - Your Langfuse public key
/// * `secret_key` - Your Langfuse secret key
///
/// # Returns
///
/// Returns a Result containing the TracerProvider if successful.
///
/// # Example
///
/// ```no_run
/// use opentelemetry_langfuse::init_tracer;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let tracer_provider = init_tracer(
///     "my-service",
///     "https://cloud.langfuse.com",
///     "pk-lf-1234567890",
///     "sk-lf-1234567890"
/// )?;
/// // Your application code here
/// # Ok(())
/// # }
/// ```
pub fn init_tracer(
    service_name: impl Into<String>,
    host: &str,
    public_key: &str,
    secret_key: &str,
) -> Result<TracerProvider> {
    TracerBuilder::new(service_name)
        .with_host(host)
        .with_credentials(public_key, secret_key)
        .install()
}
