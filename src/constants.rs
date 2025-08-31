//! Constants for the opentelemetry-langfuse library.

pub use opentelemetry_otlp::{
    OTEL_EXPORTER_OTLP_COMPRESSION, OTEL_EXPORTER_OTLP_ENDPOINT, OTEL_EXPORTER_OTLP_HEADERS,
    OTEL_EXPORTER_OTLP_TIMEOUT, OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
    OTEL_EXPORTER_OTLP_TRACES_HEADERS,
};

/// Environment variable name for the Langfuse public key.
pub const ENV_LANGFUSE_PUBLIC_KEY: &str = "LANGFUSE_PUBLIC_KEY";

/// Environment variable name for the Langfuse secret key.
pub const ENV_LANGFUSE_SECRET_KEY: &str = "LANGFUSE_SECRET_KEY";

/// Environment variable name for the Langfuse host URL.
pub const ENV_LANGFUSE_HOST: &str = "LANGFUSE_HOST";

/// Default Langfuse cloud host URL.
pub const DEFAULT_LANGFUSE_HOST: &str = "https://cloud.langfuse.com";
