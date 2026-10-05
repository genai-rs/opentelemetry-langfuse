//! Error types for the opentelemetry-langfuse library.

use opentelemetry_sdk::error::OTelSdkError;
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

    /// OpenTelemetry SDK error.
    #[error("OpenTelemetry error: {0}")]
    OpenTelemetry(#[from] OTelSdkError),

    /// OTLP exporter build error.
    #[error("OTLP exporter error: {0}")]
    OtlpExporter(#[from] opentelemetry_otlp::ExporterBuildError),
}

/// Result type alias for opentelemetry-langfuse operations.
pub type Result<T> = std::result::Result<T, Error>;
