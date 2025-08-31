//! OpenTelemetry integration for Langfuse observability.
//!
//! This crate provides a bridge between OpenTelemetry and Langfuse,
//! allowing you to export OpenTelemetry traces to the Langfuse platform
//! for LLM observability and monitoring.
//!
//! # Quick Start
//!
//! ```no_run
//! use opentelemetry_langfuse::init_tracer_from_env;
//! use opentelemetry::global;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize tracer from environment variables
//! // Requires: LANGFUSE_HOST, LANGFUSE_PUBLIC_KEY, LANGFUSE_SECRET_KEY
//! let _tracer_provider = init_tracer_from_env("my-service")?;
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
//! - Easy integration with Langfuse via OpenTelemetry
//! - Support for environment variable configuration
//! - Builder pattern for custom configuration
//! - Automatic OTLP/HTTP export to Langfuse

pub mod auth;
pub mod constants;
pub mod endpoint;
pub mod error;
pub mod tracer;

// Re-export main types
pub use auth::{build_auth_header, build_auth_header_from_env};
pub use endpoint::{build_otlp_endpoint, build_otlp_endpoint_from_env};
pub use error::{Error, Result};
pub use tracer::{init_tracer, init_tracer_from_env, TracerBuilder};
