//! OpenTelemetry integration for Langfuse.
//!
//! This crate provides OpenTelemetry components and utilities for integrating
//! with Langfuse, enabling comprehensive observability for LLM applications.
//!
//! For detailed information about OpenTelemetry support in Langfuse, see the
//! [official documentation](https://langfuse.com/integrations/native/opentelemetry).
//!
//! # Quick Start
//!
//! ```no_run
//! use opentelemetry_langfuse::ExporterBuilder;
//! use opentelemetry_sdk::trace::{
//!     span_processor_with_async_runtime::BatchSpanProcessor,
//!     SdkTracerProvider,
//! };
//! use opentelemetry_sdk::{runtime::Tokio, Resource};
//! use opentelemetry::KeyValue;
//! use opentelemetry::global;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create the Langfuse exporter from environment variables
//! // Requires: LANGFUSE_HOST, LANGFUSE_PUBLIC_KEY, LANGFUSE_SECRET_KEY
//! let exporter = ExporterBuilder::from_env()?.build()?;
//!
//! // Create your tracer provider with the Langfuse exporter
//! let provider = SdkTracerProvider::builder()
//!     .with_span_processor(BatchSpanProcessor::builder(exporter, Tokio).build())
//!     .with_resource(Resource::builder().with_attributes(vec![
//!         KeyValue::new("service.name", "my-service"),
//!     ]).build())
//!     .build();
//!
//! // Set as global provider
//! global::set_tracer_provider(provider);
//!
//! // Use the tracer
//! let tracer = global::tracer("my-tracer");
//! // ... your tracing code here ...
//!
//! // Provider will be shutdown when it goes out of scope
//! # Ok(())
//! # }
//! ```
//!
//! # Components
//!
//! ## Exporter
//! - Configured OTLP/HTTP exporter for sending traces to Langfuse
//! - Automatic authentication header setup
//! - Environment variable configuration support
//! - Builder pattern for custom configuration
//!
//! # Environment Variables
//!
//! This crate uses Langfuse-specific environment variables for configuration:
//!
//! - `LANGFUSE_HOST`: Base URL of your Langfuse instance (defaults to `https://cloud.langfuse.com`)
//! - `LANGFUSE_PUBLIC_KEY`: Your Langfuse public key (required)
//! - `LANGFUSE_SECRET_KEY`: Your Langfuse secret key (required)
//!
//! Example:
//! ```bash
//! export LANGFUSE_HOST="https://cloud.langfuse.com"
//! export LANGFUSE_PUBLIC_KEY="pk-lf-..."
//! export LANGFUSE_SECRET_KEY="sk-lf-..."
//! ```
//!
//! Use `ExporterBuilder::from_env()` to create an exporter using these variables.

pub mod auth;
pub mod constants;
pub mod context;
pub mod endpoint;
pub mod error;
pub mod exporter;

// Re-export main types
pub use auth::{build_auth_header, build_auth_header_from_env};
pub use context::{LangfuseContext, LangfuseContextBuilder, GLOBAL_CONTEXT};
pub use endpoint::{build_otlp_endpoint, build_otlp_endpoint_from_env};
pub use error::{Error, Result};
pub use exporter::{exporter, ExporterBuilder};
