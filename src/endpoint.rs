//! Endpoint URL utilities for Langfuse.

use crate::constants::{DEFAULT_LANGFUSE_HOST, ENV_LANGFUSE_HOST};
use std::env;

/// Builds the Langfuse OTLP endpoint URL by appending the API path.
///
/// This function takes a base URL and appends "/api/public/otel" to create
/// the full OTLP endpoint URL for Langfuse.
///
/// # Arguments
///
/// * `base_url` - The base Langfuse URL (e.g., `https://cloud.langfuse.com`)
///
/// # Returns
///
/// Returns the complete OTLP endpoint URL.
///
/// # Example
///
/// ```
/// use opentelemetry_langfuse::endpoint::build_otlp_endpoint;
///
/// let endpoint = build_otlp_endpoint("https://cloud.langfuse.com");
/// assert_eq!(endpoint, "https://cloud.langfuse.com/api/public/otel");
/// ```
pub fn build_otlp_endpoint(base_url: &str) -> String {
    let url = base_url.trim_end_matches('/');
    format!("{}/api/public/otel", url)
}

/// Builds the Langfuse OTLP endpoint URL from environment variable.
///
/// This function reads the LANGFUSE_HOST environment variable and creates
/// the complete OTLP endpoint URL by appending "/api/public/otel".
/// If LANGFUSE_HOST is not set, defaults to the cloud instance.
///
/// # Returns
///
/// Returns the complete OTLP endpoint URL.
///
/// # Example
///
/// ```no_run
/// use opentelemetry_langfuse::endpoint::build_otlp_endpoint_from_env;
///
/// // Uses LANGFUSE_HOST env var if set, otherwise defaults to cloud
/// let endpoint = build_otlp_endpoint_from_env().unwrap();
/// ```
pub fn build_otlp_endpoint_from_env() -> Result<String, crate::Error> {
    let base_url =
        env::var(ENV_LANGFUSE_HOST).unwrap_or_else(|_| DEFAULT_LANGFUSE_HOST.to_string());
    Ok(build_otlp_endpoint(&base_url))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_build_otlp_endpoint() {
        // Test with URL without trailing slash
        let endpoint = build_otlp_endpoint("https://cloud.langfuse.com");
        assert_eq!(endpoint, "https://cloud.langfuse.com/api/public/otel");

        // Test with URL with trailing slash
        let endpoint = build_otlp_endpoint("https://cloud.langfuse.com/");
        assert_eq!(endpoint, "https://cloud.langfuse.com/api/public/otel");

        // Test with US region URL
        let endpoint = build_otlp_endpoint("https://us.cloud.langfuse.com");
        assert_eq!(endpoint, "https://us.cloud.langfuse.com/api/public/otel");
    }

    #[test]
    #[serial]
    fn test_build_otlp_endpoint_from_env() {
        env::set_var(ENV_LANGFUSE_HOST, "https://cloud.langfuse.com");
        let endpoint = build_otlp_endpoint_from_env().unwrap();
        assert_eq!(endpoint, "https://cloud.langfuse.com/api/public/otel");

        // Test with trailing slash in env var
        env::set_var(ENV_LANGFUSE_HOST, "https://cloud.langfuse.com/");
        let endpoint = build_otlp_endpoint_from_env().unwrap();
        assert_eq!(endpoint, "https://cloud.langfuse.com/api/public/otel");

        // Cleanup
        env::remove_var(ENV_LANGFUSE_HOST);
    }

    #[test]
    #[serial]
    fn test_default_langfuse_host() {
        env::remove_var(ENV_LANGFUSE_HOST);
        let result = build_otlp_endpoint_from_env();
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "https://cloud.langfuse.com/api/public/otel"
        );
    }
}
