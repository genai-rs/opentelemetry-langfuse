//! Error types for the opentelemetry-langfuse library.

use thiserror::Error;

/// Error type for opentelemetry-langfuse operations.
#[derive(Debug, Error)]
pub enum Error {
    /// Environment variable is missing.
    #[error("Missing environment variable: {0}")]
    MissingEnvironmentVariable(&'static str),

    /// Required configuration is missing.
    #[error("Missing configuration: {0}")]
    MissingConfiguration(&'static str),

    /// OpenTelemetry trace error.
    #[error("OpenTelemetry error: {0}")]
    OpenTelemetry(#[from] opentelemetry::trace::TraceError),

    /// OTLP exporter error.
    #[error("OTLP exporter error: {0}")]
    OtlpExporter(#[from] opentelemetry_otlp::Error),
}

/// Result type alias for opentelemetry-langfuse operations.
pub type Result<T> = std::result::Result<T, Error>;
