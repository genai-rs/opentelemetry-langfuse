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
//! use opentelemetry_langfuse::exporter_from_env;
//! use opentelemetry_sdk::trace::SdkTracerProvider;
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
//! let provider = SdkTracerProvider::builder()
//!     .with_batch_exporter(exporter)
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
//! - Environment variable configuration support (both Langfuse and OTEL standards)
//! - Builder pattern for custom configuration
//!
//! # Environment Variables
//!
//! This crate supports both Langfuse-specific and standard OpenTelemetry environment variables
//! for configuration. You can choose the configuration style that best fits your needs:
//!
//! ## Langfuse-Specific Variables
//!
//! Use these when working directly with Langfuse:
//!
//! - `LANGFUSE_HOST`: Base URL of your Langfuse instance (defaults to `https://cloud.langfuse.com`)
//! - `LANGFUSE_PUBLIC_KEY`: Your Langfuse public key
//! - `LANGFUSE_SECRET_KEY`: Your Langfuse secret key
//!
//! Example:
//! ```bash
//! export LANGFUSE_HOST="https://cloud.langfuse.com"
//! export LANGFUSE_PUBLIC_KEY="pk-lf-..."
//! export LANGFUSE_SECRET_KEY="sk-lf-..."
//! ```
//!
//! Use `exporter_from_langfuse_env()` to create an exporter using only these variables.
//!
//! ## Standard OpenTelemetry Variables
//!
//! Following the [OpenTelemetry Protocol Exporter specification](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/protocol/exporter.md#endpoint-urls-for-otlphttp):
//!
//! - `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT`: Direct endpoint for traces
//! - `OTEL_EXPORTER_OTLP_ENDPOINT`: Base endpoint (will append `/v1/traces`)
//! - `OTEL_EXPORTER_OTLP_TRACES_HEADERS`: Headers for traces endpoint
//! - `OTEL_EXPORTER_OTLP_HEADERS`: General headers
//!
//! Example:
//! ```bash
//! export OTEL_EXPORTER_OTLP_TRACES_ENDPOINT="https://cloud.langfuse.com/api/public/otel"
//! export OTEL_EXPORTER_OTLP_TRACES_HEADERS="Authorization=Basic <base64_encoded_credentials>"
//! ```
//!
//! Use `exporter_from_otel_env()` to create an exporter using only these variables.
//!
//! ## Automatic Fallback
//!
//! The `exporter_from_env()` function provides automatic fallback between both styles,
//! with Langfuse-specific variables taking precedence:
//!
//! 1. First checks for Langfuse-specific variables
//! 2. Falls back to standard OTEL variables if Langfuse variables are not found
//! 3. Uses sensible defaults where applicable
//!
//! This allows for flexible configuration in different deployment scenarios.

pub mod auth;
pub mod constants;
pub mod endpoint;
pub mod error;
pub mod exporter;

// Re-export main types
pub use auth::{build_auth_header, build_auth_header_from_env};
pub use endpoint::{build_otlp_endpoint, build_otlp_endpoint_from_env};
pub use error::{Error, Result};
pub use exporter::{
    exporter, exporter_from_env, exporter_from_langfuse_env, exporter_from_otel_env,
    ExporterBuilder,
};
