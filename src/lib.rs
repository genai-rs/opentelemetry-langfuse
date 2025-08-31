//! OpenTelemetry Langfuse exporter.
//!
//! This crate provides a configured OTLP exporter for sending OpenTelemetry
//! traces to Langfuse for LLM observability and monitoring.
//!
//! # Quick Start
//!
//! ```no_run
//! use opentelemetry_langfuse::exporter_from_env;
//! use opentelemetry_sdk::trace::TracerProvider;
//! use opentelemetry_sdk::Resource;
//! use opentelemetry::KeyValue;
//! use opentelemetry::global;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create the Langfuse exporter from environment variables
//! // Requires: LANGFUSE_HOST, LANGFUSE_PUBLIC_KEY, LANGFUSE_SECRET_KEY
//! let exporter = exporter_from_env()?;
//!
//! // Create your tracer provider with the Langfuse exporter
//! let provider = TracerProvider::builder()
//!     .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
//!     .with_resource(Resource::new(vec![
//!         KeyValue::new("service.name", "my-service"),
//!     ]))
//!     .build();
//!
//! // Set as global provider
//! global::set_tracer_provider(provider);
//!
//! // Use the tracer
//! let tracer = global::tracer("my-tracer");
//! // ... your tracing code here ...
//!
//! // Shutdown when done
//! global::shutdown_tracer_provider();
//! # Ok(())
//! # }
//! ```
//!
//! # Features
//!
//! - Configured OTLP/HTTP exporter for Langfuse
//! - Automatic authentication header setup
//! - Environment variable configuration support
//! - Builder pattern for custom configuration

pub mod auth;
pub mod constants;
pub mod endpoint;
pub mod error;
pub mod exporter;

// Re-export main types
pub use auth::{build_auth_header, build_auth_header_from_env};
pub use endpoint::{build_otlp_endpoint, build_otlp_endpoint_from_env};
pub use error::{Error, Result};
pub use exporter::{exporter, exporter_from_env, ExporterBuilder};
