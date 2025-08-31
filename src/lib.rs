//! OpenTelemetry integration for Langfuse.
//!
//! This crate provides OpenTelemetry components and utilities for integrating
//! with Langfuse, enabling comprehensive observability for LLM applications.
//!
//! # Features
//!
//! - **Attribute Mapping**: Bidirectional mapping between Langfuse and OpenTelemetry GenAI conventions
//! - **Context Management**: Thread-safe, explicit context passing without global state
//! - **Custom Span Processing**: Enriches spans with Langfuse-specific attributes
//! - **Builder Pattern**: Fluent API for configuring tracers and contexts
//! - **GenAI Support**: Full support for OpenTelemetry GenAI semantic conventions
//!
//! # Quick Start
//!
//! ```no_run
//! use opentelemetry_langfuse::{builder, TracingContext};
//! use opentelemetry::trace::Tracer;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a context for your traces
//! let context = TracingContext::new()
//!     .with_session("session-123")
//!     .with_user("user-456")
//!     .with_metadata("environment", serde_json::json!("production"));
//!
//! // Build a tracer with Langfuse integration
//! let tracer = builder()
//!     .with_service_name("my-llm-service")
//!     .with_api_key("your-langfuse-api-key")
//!     .with_context(context)
//!     .build()?;
//!
//! // Use the tracer for LLM operations
//! let mut span = tracer.span_builder("chat.completion").start(&tracer);
//! span.set_attribute(opentelemetry::KeyValue::new("gen_ai.request.model", "gpt-4"));
//! // ... your LLM call here ...
//! span.end();
//! # Ok(())
//! # }
//! ```
//!
//! # Context Management
//!
//! The library provides explicit context passing, avoiding global state:
//!
//! ```no_run
//! use opentelemetry_langfuse::{TracingContext, TracingContextBuilder};
//!
//! // Create a context using the builder
//! let context = TracingContextBuilder::new()
//!     .session_id("session-789")
//!     .user_id("user-012")
//!     .model("gpt-4")
//!     .temperature(0.7)
//!     .build();
//!
//! // Pass context explicitly through your application
//! fn process_request(context: &TracingContext) {
//!     // Context is available without global state
//!     let session = context.get_attribute("session.id");
//! }
//! ```
//!
//! # Attribute Mapping
//!
//! Automatic mapping between Langfuse and OpenTelemetry conventions:
//!
//! ```no_run
//! use opentelemetry_langfuse::mapper::{GenAIAttributeMapper, AttributeMapper};
//! use opentelemetry::KeyValue;
//!
//! let mapper = GenAIAttributeMapper::new();
//!
//! // OpenTelemetry attributes
//! let otel_attrs = vec![
//!     KeyValue::new("gen_ai.request.model", "gpt-4"),
//!     KeyValue::new("gen_ai.usage.prompt_tokens", 150i64),
//! ];
//!
//! // Automatically mapped to Langfuse format
//! let langfuse_attrs = mapper.map_to_langfuse(&otel_attrs);
//! ```

// New modules for the integration
pub mod attributes;
pub mod builder;
pub mod context;
pub mod mapper;
pub mod processor;

// Existing modules
pub mod auth;
pub mod constants;
pub mod endpoint;
pub mod error;
pub mod exporter;

// Re-export main types for the new integration
pub use attributes::{
    LangfuseAttributes, ObservationAttributesBuilder, OpenTelemetryGenAIAttributes,
    TraceAttributesBuilder,
};
pub use builder::{BatchConfig, BuilderError, BuilderResult, LangfuseTracerBuilder};
pub use context::{TracingContext, TracingContextBuilder};
pub use mapper::{AttributeMapper, GenAIAttributeMapper, MappingRule, PassThroughMapper};
pub use processor::{LangfuseSpanProcessor, MappingExporter};

// Re-export existing types
pub use auth::{build_auth_header, build_auth_header_from_env};
pub use endpoint::{build_otlp_endpoint, build_otlp_endpoint_from_env};
pub use error::{Error, Result};
pub use exporter::{
    exporter, exporter_from_env, exporter_from_langfuse_env, exporter_from_otel_env,
    ExporterBuilder,
};

// Convenience re-export for Tokio runtime
#[cfg(feature = "tokio")]
pub use builder::builder;
