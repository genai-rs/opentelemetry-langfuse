//! Constants for the opentelemetry-langfuse library.

/// Environment variable name for the Langfuse public key.
pub const ENV_LANGFUSE_PUBLIC_KEY: &str = "LANGFUSE_PUBLIC_KEY";

/// Environment variable name for the Langfuse secret key.
pub const ENV_LANGFUSE_SECRET_KEY: &str = "LANGFUSE_SECRET_KEY";

/// Environment variable name for the Langfuse host URL.
pub const ENV_LANGFUSE_HOST: &str = "LANGFUSE_HOST";

/// Default Langfuse cloud host URL.
pub const DEFAULT_LANGFUSE_HOST: &str = "https://cloud.langfuse.com";

/// Environment variable name for standard OTLP traces endpoint.
pub const ENV_OTEL_EXPORTER_OTLP_TRACES_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT";

/// Environment variable name for standard OTLP endpoint.
pub const ENV_OTEL_EXPORTER_OTLP_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_ENDPOINT";

/// Environment variable name for standard OTLP headers.
pub const ENV_OTEL_EXPORTER_OTLP_HEADERS: &str = "OTEL_EXPORTER_OTLP_HEADERS";

/// Environment variable name for standard OTLP traces headers.
pub const ENV_OTEL_EXPORTER_OTLP_TRACES_HEADERS: &str = "OTEL_EXPORTER_OTLP_TRACES_HEADERS";