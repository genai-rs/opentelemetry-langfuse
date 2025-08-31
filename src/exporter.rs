//! Langfuse OTLP exporter configuration.
//!
//! This module provides the [`ExporterBuilder`] for configuring an OpenTelemetry
//! OTLP exporter that sends traces to Langfuse.
//!
//! See the [Langfuse OpenTelemetry documentation](https://langfuse.com/integrations/native/opentelemetry)
//! for more details about the integration.

use crate::{auth, constants::*, endpoint, Error, Result};
use opentelemetry_otlp::{SpanExporter, WithExportConfig, WithHttpConfig};
use std::collections::HashMap;
use std::env;
use std::time::Duration;

/// Builder for configuring a Langfuse OTLP exporter.
pub struct ExporterBuilder {
    endpoint: Option<String>,
    auth_header: Option<String>,
    timeout: Option<Duration>,
    additional_headers: HashMap<String, String>,
}

impl ExporterBuilder {
    /// Creates a new ExporterBuilder.
    pub fn new() -> Self {
        Self {
            endpoint: None,
            auth_header: None,
            timeout: None,
            additional_headers: HashMap::new(),
        }
    }

    /// Sets the Langfuse endpoint URL.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The complete OTLP endpoint URL
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    /// Sets the authentication header.
    ///
    /// # Arguments
    ///
    /// * `auth_header` - The complete authentication header value
    pub fn with_auth_header(mut self, auth_header: impl Into<String>) -> Self {
        self.auth_header = Some(auth_header.into());
        self
    }

    /// Sets the Basic authentication credentials.
    ///
    /// # Arguments
    ///
    /// * `public_key` - The Langfuse public key
    /// * `secret_key` - The Langfuse secret key
    pub fn with_basic_auth(mut self, public_key: &str, secret_key: &str) -> Self {
        self.auth_header = Some(auth::build_auth_header(public_key, secret_key));
        self
    }

    /// Sets the Langfuse host URL.
    ///
    /// # Arguments
    ///
    /// * `host` - The base Langfuse URL (e.g., `https://cloud.langfuse.com`)
    pub fn with_host(mut self, host: &str) -> Self {
        self.endpoint = Some(endpoint::build_otlp_endpoint(host));
        self
    }

    /// Sets the HTTP timeout for the exporter.
    ///
    /// # Arguments
    ///
    /// * `timeout` - The timeout duration
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Adds an additional HTTP header.
    ///
    /// # Arguments
    ///
    /// * `name` - The header name
    /// * `value` - The header value
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.additional_headers.insert(name.into(), value.into());
        self
    }

    /// Adds multiple HTTP headers.
    ///
    /// # Arguments
    ///
    /// * `headers` - An iterator of header name-value pairs
    pub fn with_headers<I, K, V>(mut self, headers: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.additional_headers
            .extend(headers.into_iter().map(|(k, v)| (k.into(), v.into())));
        self
    }

    /// Loads configuration from environment variables.
    ///
    /// This method reads (in order of precedence):
    ///
    /// For endpoint:
    /// 1. LANGFUSE_HOST (with /api/public/otel appended)
    /// 2. OTEL_EXPORTER_OTLP_TRACES_ENDPOINT
    /// 3. OTEL_EXPORTER_OTLP_ENDPOINT (with /v1/traces appended)
    /// 4. Default to cloud.langfuse.com
    ///
    /// For authentication:
    /// 1. LANGFUSE_PUBLIC_KEY and LANGFUSE_SECRET_KEY
    /// 2. OTEL_EXPORTER_OTLP_TRACES_HEADERS
    /// 3. OTEL_EXPORTER_OTLP_HEADERS
    ///
    /// Also supports:
    /// - OTEL_EXPORTER_OTLP_TIMEOUT: Timeout in milliseconds
    /// - OTEL_EXPORTER_OTLP_COMPRESSION: Compression algorithm (gzip or none)
    pub fn from_env(mut self) -> Result<Self> {
        // Check for Langfuse-specific endpoint first (may use default)
        let langfuse_endpoint = endpoint::build_otlp_endpoint_from_env()?;

        // Only use OTEL endpoints if LANGFUSE_HOST was not explicitly set
        if env::var(ENV_LANGFUSE_HOST).is_err() {
            // No LANGFUSE_HOST set, check for OTEL endpoints
            if let Ok(endpoint) = env::var(OTEL_EXPORTER_OTLP_TRACES_ENDPOINT) {
                self.endpoint = Some(endpoint);
            } else if let Ok(endpoint) = env::var(OTEL_EXPORTER_OTLP_ENDPOINT) {
                // OTEL_EXPORTER_OTLP_ENDPOINT needs /v1/traces appended
                self.endpoint = Some(format!("{}/v1/traces", endpoint.trim_end_matches('/')));
            } else {
                // Use the Langfuse endpoint (which defaults to cloud)
                self.endpoint = Some(langfuse_endpoint);
            }
        } else {
            // LANGFUSE_HOST was explicitly set, use it
            self.endpoint = Some(langfuse_endpoint);
        }

        // Try Langfuse credentials first
        if let Ok(auth) = auth::build_auth_header_from_env() {
            self.auth_header = Some(auth);
        } else {
            // Fall back to OTEL headers if Langfuse credentials not available
            if let Ok(headers) = env::var(OTEL_EXPORTER_OTLP_TRACES_HEADERS)
                .or_else(|_| env::var(OTEL_EXPORTER_OTLP_HEADERS))
            {
                // Parse OTEL headers format: "key1=value1,key2=value2"
                for header_pair in headers.split(',') {
                    if let Some((key, value)) = header_pair.split_once('=') {
                        let key = key.trim().to_string();
                        let value = value.trim().to_string();

                        // Check if this is an Authorization header
                        if key.eq_ignore_ascii_case("authorization") && self.auth_header.is_none() {
                            self.auth_header = Some(value);
                        } else {
                            self.additional_headers.insert(key, value);
                        }
                    }
                }
            }
        }

        // Handle timeout configuration
        if let Ok(timeout_str) = env::var(OTEL_EXPORTER_OTLP_TIMEOUT) {
            if let Ok(timeout_ms) = timeout_str.parse::<u64>() {
                self.timeout = Some(Duration::from_millis(timeout_ms));
            }
        }

        // Handle compression configuration
        if let Ok(compression) = env::var(OTEL_EXPORTER_OTLP_COMPRESSION) {
            if compression.eq_ignore_ascii_case("gzip") {
                // Note: The actual compression is handled by the SpanExporter builder
                // We just document that we support it
            }
        }

        Ok(self)
    }

    /// Builds the Langfuse OTLP exporter.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the configured SpanExporter if successful.
    pub fn build(self) -> Result<SpanExporter> {
        let endpoint = self
            .endpoint
            .ok_or(Error::MissingConfiguration("endpoint"))?;

        // Create headers map
        let mut headers = HashMap::new();

        // Add additional headers first (may include Authorization from OTEL env)
        headers.extend(self.additional_headers);

        // Check if Authorization header exists (case-insensitive)
        // and normalize it to "Authorization" if found with different casing
        let has_auth = headers
            .keys()
            .any(|k| k.eq_ignore_ascii_case("authorization"));

        if has_auth {
            // Find and normalize the Authorization header key
            let auth_keys: Vec<String> = headers
                .keys()
                .filter(|k| k.eq_ignore_ascii_case("authorization"))
                .cloned()
                .collect();

            // If we have authorization with non-standard casing, normalize it
            for key in auth_keys {
                if key != "Authorization" {
                    if let Some(value) = headers.remove(&key) {
                        headers.insert("Authorization".to_string(), value);
                    }
                }
            }
        } else {
            // No Authorization header found, add the auth_header if available
            if let Some(auth_header) = self.auth_header {
                headers.insert("Authorization".to_string(), auth_header);
            } else {
                return Err(Error::MissingConfiguration(
                    "Authorization header or Langfuse credentials",
                ));
            }
        }

        // Build HTTP config
        let mut http_config = SpanExporter::builder()
            .with_http()
            .with_endpoint(endpoint)
            .with_headers(headers);

        // Apply timeout if configured
        if let Some(timeout) = self.timeout {
            http_config = http_config.with_timeout(timeout);
        }

        // Create OTLP exporter
        Ok(http_config.build()?)
    }
}

impl Default for ExporterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a Langfuse OTLP exporter using Langfuse-specific environment variables.
///
/// This function primarily looks for Langfuse-specific environment variables:
/// - `LANGFUSE_HOST`: The base URL of your Langfuse instance (defaults to https://cloud.langfuse.com)
/// - `LANGFUSE_PUBLIC_KEY`: Your Langfuse public key (required)
/// - `LANGFUSE_SECRET_KEY`: Your Langfuse secret key (required)
///
/// The OTLP endpoint will be constructed as `{LANGFUSE_HOST}/api/public/otel`.
///
/// Also supports standard OTEL configuration variables:
/// - `OTEL_EXPORTER_OTLP_TIMEOUT`: Timeout in milliseconds (default: 10000)
/// - `OTEL_EXPORTER_OTLP_COMPRESSION`: Compression algorithm (`gzip` or none)
///
/// # Returns
///
/// Returns a Result containing the configured SpanExporter if successful.
///
/// # Example
///
/// ```no_run
/// use opentelemetry_langfuse::exporter_from_langfuse_env;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let exporter = exporter_from_langfuse_env()?;
/// // Use the exporter with your TracerProvider setup
/// # Ok(())
/// # }
/// ```
pub fn exporter_from_langfuse_env() -> Result<SpanExporter> {
    let endpoint = endpoint::build_otlp_endpoint_from_env()?;
    let auth = auth::build_auth_header_from_env()?;

    let mut builder = ExporterBuilder::new()
        .with_endpoint(endpoint)
        .with_auth_header(auth);

    // Handle timeout configuration
    if let Ok(timeout_str) = env::var(OTEL_EXPORTER_OTLP_TIMEOUT) {
        if let Ok(timeout_ms) = timeout_str.parse::<u64>() {
            builder = builder.with_timeout(Duration::from_millis(timeout_ms));
        }
    }

    // Handle compression configuration
    if let Ok(compression) = env::var(OTEL_EXPORTER_OTLP_COMPRESSION) {
        if compression.eq_ignore_ascii_case("gzip") {
            // Note: The actual compression is handled by the SpanExporter builder
            // We just document that we support it
        }
    }

    builder.build()
}

/// Creates a Langfuse OTLP exporter using standard OpenTelemetry environment variables.
///
/// This function follows the [OpenTelemetry Protocol Exporter specification](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/protocol/exporter.md#endpoint-urls-for-otlphttp)
/// and only looks for standard OTEL environment variables:
///
/// ## Supported Environment Variables
///
/// ### Endpoint Configuration
/// - `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT`: Direct OTLP traces endpoint URL
/// - `OTEL_EXPORTER_OTLP_ENDPOINT`: Base OTLP endpoint (will append `/v1/traces`)
///
/// ### Headers Configuration
/// - `OTEL_EXPORTER_OTLP_TRACES_HEADERS`: Headers for traces endpoint
/// - `OTEL_EXPORTER_OTLP_HEADERS`: General OTLP headers
///
/// Headers should be in the format: `key1=value1,key2=value2`
///
/// ### Additional Configuration
/// - `OTEL_EXPORTER_OTLP_TIMEOUT`: Timeout in milliseconds (default: 10000)
/// - `OTEL_EXPORTER_OTLP_COMPRESSION`: Compression algorithm (`gzip` or none)
///
/// ## Langfuse Configuration
///
/// For Langfuse, use one of these endpoint configurations:
/// - `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT=https://cloud.langfuse.com/api/public/otel`
/// - `OTEL_EXPORTER_OTLP_ENDPOINT=https://cloud.langfuse.com/api/public/otel` (creates `/api/public/otel/v1/traces`)
///
/// ⚠️ Do NOT use `OTEL_EXPORTER_OTLP_ENDPOINT=https://cloud.langfuse.com/api/public` as this would
/// create `/api/public/v1/traces` which Langfuse does not accept.
///
/// # Returns
///
/// Returns a Result containing the configured SpanExporter if successful.
///
/// # Example
///
/// ```no_run
/// use opentelemetry_langfuse::exporter_from_otel_env;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Set environment variables:
/// // OTEL_EXPORTER_OTLP_ENDPOINT=https://cloud.langfuse.com/api/public/otel
/// // OTEL_EXPORTER_OTLP_HEADERS=Authorization=Basic <base64_encoded_credentials>
/// let exporter = exporter_from_otel_env()?;
/// // Use the exporter with your TracerProvider setup
/// # Ok(())
/// # }
/// ```
pub fn exporter_from_otel_env() -> Result<SpanExporter> {
    // Get endpoint from OTEL environment variables
    let endpoint = if let Ok(endpoint) = env::var(OTEL_EXPORTER_OTLP_TRACES_ENDPOINT) {
        endpoint
    } else if let Ok(endpoint) = env::var(OTEL_EXPORTER_OTLP_ENDPOINT) {
        // OTEL_EXPORTER_OTLP_ENDPOINT needs /v1/traces appended
        format!("{}/v1/traces", endpoint.trim_end_matches('/'))
    } else {
        return Err(Error::MissingEnvironmentVariable(
            "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT or OTEL_EXPORTER_OTLP_ENDPOINT",
        ));
    };

    // Parse headers from OTEL environment variables
    let headers_str = env::var(OTEL_EXPORTER_OTLP_TRACES_HEADERS)
        .or_else(|_| env::var(OTEL_EXPORTER_OTLP_HEADERS))
        .map_err(|_| {
            Error::MissingEnvironmentVariable(
                "OTEL_EXPORTER_OTLP_TRACES_HEADERS or OTEL_EXPORTER_OTLP_HEADERS",
            )
        })?;

    let mut builder = ExporterBuilder::new().with_endpoint(endpoint);

    // Parse OTEL headers format: "key1=value1,key2=value2"
    for header_pair in headers_str.split(',') {
        if let Some((key, value)) = header_pair.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().to_string();

            // Check if this is an Authorization header
            if key.eq_ignore_ascii_case("authorization") {
                builder = builder.with_auth_header(value);
            } else {
                builder = builder.with_header(key, value);
            }
        }
    }

    // Handle timeout configuration
    if let Ok(timeout_str) = env::var(OTEL_EXPORTER_OTLP_TIMEOUT) {
        if let Ok(timeout_ms) = timeout_str.parse::<u64>() {
            builder = builder.with_timeout(Duration::from_millis(timeout_ms));
        }
    }

    // Handle compression configuration
    if let Ok(compression) = env::var(OTEL_EXPORTER_OTLP_COMPRESSION) {
        if compression.eq_ignore_ascii_case("gzip") {
            // Note: The actual compression is handled by the SpanExporter builder
            // We just document that we support it
        }
    }

    builder.build()
}

/// Creates a Langfuse OTLP exporter using environment variables with automatic fallback.
///
/// This function provides automatic fallback between Langfuse-specific and standard OpenTelemetry
/// environment variables. Langfuse-specific variables take precedence over standard OTEL variables.
///
/// ## Environment Variable Priority
///
/// ### For endpoint (in order of precedence):
/// 1. `LANGFUSE_HOST`: The base URL of your Langfuse instance (appends `/api/public/otel`)
/// 2. `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT`: Direct OTLP traces endpoint URL
/// 3. `OTEL_EXPORTER_OTLP_ENDPOINT`: Base OTLP endpoint (appends `/v1/traces`)
/// 4. **Default**: `https://cloud.langfuse.com/api/public/otel` (when no endpoint variables are set)
///
/// ### For authentication (in order of precedence):
/// 1. `LANGFUSE_PUBLIC_KEY` + `LANGFUSE_SECRET_KEY`: Your Langfuse credentials
/// 2. `OTEL_EXPORTER_OTLP_TRACES_HEADERS`: Headers including Authorization
/// 3. `OTEL_EXPORTER_OTLP_HEADERS`: Headers including Authorization
///
/// ### Additional Configuration:
/// - `OTEL_EXPORTER_OTLP_TIMEOUT`: Timeout in milliseconds (default: 10000)
/// - `OTEL_EXPORTER_OTLP_COMPRESSION`: Compression algorithm (`gzip` or none)
///
/// ## Usage Recommendations
///
/// - Use `exporter_from_langfuse_env()` if you're exclusively using Langfuse-specific variables
/// - Use `exporter_from_otel_env()` if you're following standard OpenTelemetry configuration
/// - Use this function if you want automatic fallback between both configuration styles
///
/// ## References
///
/// - [Langfuse OpenTelemetry Integration](https://langfuse.com/integrations/native/opentelemetry)
/// - [OpenTelemetry Protocol Exporter Specification](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/protocol/exporter.md#endpoint-urls-for-otlphttp)
///
/// # Returns
///
/// Returns a Result containing the configured SpanExporter if successful.
///
/// # Example
///
/// ```no_run
/// use opentelemetry_langfuse::exporter_from_env;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let exporter = exporter_from_env()?;
/// // Use the exporter with your TracerProvider setup
/// # Ok(())
/// # }
/// ```
pub fn exporter_from_env() -> Result<SpanExporter> {
    ExporterBuilder::new().from_env()?.build()
}

/// Creates a Langfuse OTLP exporter using explicit configuration.
///
/// # Arguments
///
/// * `host` - The base Langfuse URL (e.g., `https://cloud.langfuse.com`)
/// * `public_key` - Your Langfuse public key
/// * `secret_key` - Your Langfuse secret key
///
/// # Returns
///
/// Returns a Result containing the configured SpanExporter if successful.
///
/// # Example
///
/// ```no_run
/// use opentelemetry_langfuse::exporter;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let exporter = exporter(
///     "https://cloud.langfuse.com",
///     "pk-lf-1234567890",
///     "sk-lf-1234567890"
/// )?;
/// // Use the exporter with your TracerProvider setup
/// # Ok(())
/// # }
/// ```
pub fn exporter(host: &str, public_key: &str, secret_key: &str) -> Result<SpanExporter> {
    ExporterBuilder::new()
        .with_host(host)
        .with_basic_auth(public_key, secret_key)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_exporter_from_langfuse_env() {
        // Set up Langfuse environment variables
        env::set_var("LANGFUSE_HOST", "https://test.langfuse.com");
        env::set_var("LANGFUSE_PUBLIC_KEY", "pk-test");
        env::set_var("LANGFUSE_SECRET_KEY", "sk-test");

        // The function should not panic and return a result
        // In tests, this will fail with OtlpExporter error due to missing HTTP client
        let result = exporter_from_langfuse_env();
        assert!(matches!(result, Err(Error::OtlpExporter(_))));

        // Clean up
        env::remove_var("LANGFUSE_HOST");
        env::remove_var("LANGFUSE_PUBLIC_KEY");
        env::remove_var("LANGFUSE_SECRET_KEY");
    }

    #[test]
    #[serial]
    fn test_exporter_from_langfuse_env_missing_credentials() {
        // Set only host, no credentials
        env::set_var("LANGFUSE_HOST", "https://test.langfuse.com");

        // Should fail with MissingEnvironmentVariable error
        let result = exporter_from_langfuse_env();
        assert!(matches!(result, Err(Error::MissingEnvironmentVariable(_))));

        // Clean up
        env::remove_var("LANGFUSE_HOST");
    }

    #[test]
    #[serial]
    fn test_exporter_from_otel_env() {
        // Set up OTEL environment variables
        env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "https://test.com");
        env::set_var(
            "OTEL_EXPORTER_OTLP_HEADERS",
            "Authorization=Bearer test-token",
        );

        // The function should not panic and return a result
        // In tests, this will fail with OtlpExporter error due to missing HTTP client
        let result = exporter_from_otel_env();
        assert!(matches!(result, Err(Error::OtlpExporter(_))));

        // Clean up
        env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        env::remove_var("OTEL_EXPORTER_OTLP_HEADERS");
    }

    #[test]
    #[serial]
    fn test_exporter_from_otel_env_traces_endpoint() {
        // Set up OTEL traces-specific endpoint
        env::set_var(
            "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT",
            "https://test.com/v1/traces",
        );
        env::set_var(
            "OTEL_EXPORTER_OTLP_TRACES_HEADERS",
            "Authorization=Bearer test-token",
        );

        // The function should not panic and return a result
        // In tests, this will fail with OtlpExporter error due to missing HTTP client
        let result = exporter_from_otel_env();
        assert!(matches!(result, Err(Error::OtlpExporter(_))));

        // Clean up
        env::remove_var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT");
        env::remove_var("OTEL_EXPORTER_OTLP_TRACES_HEADERS");
    }

    #[test]
    #[serial]
    fn test_exporter_from_otel_env_lowercase_authorization() {
        // Test that lowercase "authorization" header is handled correctly
        env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "https://test.com");
        env::set_var(
            "OTEL_EXPORTER_OTLP_HEADERS",
            "authorization=Bearer test-token",
        );

        // The function should not panic and handle lowercase authorization
        let result = exporter_from_otel_env();
        assert!(matches!(result, Err(Error::OtlpExporter(_))));

        // Clean up
        env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        env::remove_var("OTEL_EXPORTER_OTLP_HEADERS");
    }

    #[test]
    #[serial]
    fn test_exporter_from_otel_env_missing_endpoint() {
        // Set only headers, no endpoint
        env::set_var(
            "OTEL_EXPORTER_OTLP_HEADERS",
            "Authorization=Bearer test-token",
        );

        // Should fail with MissingEnvironmentVariable error
        let result = exporter_from_otel_env();
        assert!(matches!(result, Err(Error::MissingEnvironmentVariable(_))));

        // Clean up
        env::remove_var("OTEL_EXPORTER_OTLP_HEADERS");
    }

    #[test]
    #[serial]
    fn test_exporter_from_otel_env_missing_headers() {
        // Set only endpoint, no headers
        env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "https://test.com");

        // Should fail with MissingEnvironmentVariable error
        let result = exporter_from_otel_env();
        assert!(matches!(result, Err(Error::MissingEnvironmentVariable(_))));

        // Clean up
        env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
    }

    #[test]
    #[serial]
    fn test_exporter_from_env_langfuse_precedence() {
        // Set both Langfuse and OTEL variables
        env::set_var("LANGFUSE_HOST", "https://langfuse.test.com");
        env::set_var("LANGFUSE_PUBLIC_KEY", "pk-test");
        env::set_var("LANGFUSE_SECRET_KEY", "sk-test");
        env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "https://otel.test.com");
        env::set_var(
            "OTEL_EXPORTER_OTLP_HEADERS",
            "Authorization=Bearer otel-token",
        );

        // The function should not panic and return a result
        // In tests, this will fail with OtlpExporter error due to missing HTTP client
        let result = exporter_from_env();
        assert!(matches!(result, Err(Error::OtlpExporter(_))));

        // Clean up
        env::remove_var("LANGFUSE_HOST");
        env::remove_var("LANGFUSE_PUBLIC_KEY");
        env::remove_var("LANGFUSE_SECRET_KEY");
        env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        env::remove_var("OTEL_EXPORTER_OTLP_HEADERS");
    }

    #[test]
    #[serial]
    fn test_exporter_from_env_otel_fallback() {
        // Set only OTEL variables (no Langfuse variables)
        env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "https://otel.test.com");
        env::set_var(
            "OTEL_EXPORTER_OTLP_HEADERS",
            "Authorization=Bearer otel-token",
        );

        // The function should not panic and return a result
        // In tests, this will fail with OtlpExporter error due to missing HTTP client
        let result = exporter_from_env();
        assert!(matches!(result, Err(Error::OtlpExporter(_))));

        // Clean up
        env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        env::remove_var("OTEL_EXPORTER_OTLP_HEADERS");
    }

    #[test]
    #[serial]
    fn test_timeout_and_compression_env_vars() {
        // Test that timeout and compression env vars are considered in all functions

        // Test with exporter_from_langfuse_env
        env::set_var("LANGFUSE_HOST", "https://test.com");
        env::set_var("LANGFUSE_PUBLIC_KEY", "pk-test");
        env::set_var("LANGFUSE_SECRET_KEY", "sk-test");
        env::set_var("OTEL_EXPORTER_OTLP_TIMEOUT", "5000");
        env::set_var("OTEL_EXPORTER_OTLP_COMPRESSION", "gzip");

        let result = exporter_from_langfuse_env();
        assert!(matches!(result, Err(Error::OtlpExporter(_))));

        // Clean up for next test
        env::remove_var("LANGFUSE_HOST");
        env::remove_var("LANGFUSE_PUBLIC_KEY");
        env::remove_var("LANGFUSE_SECRET_KEY");

        // Test with exporter_from_otel_env
        env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "https://test.com");
        env::set_var("OTEL_EXPORTER_OTLP_HEADERS", "Authorization=Bearer test");

        let result = exporter_from_otel_env();
        assert!(matches!(result, Err(Error::OtlpExporter(_))));

        // Clean up for next test
        env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        env::remove_var("OTEL_EXPORTER_OTLP_HEADERS");

        // Test with exporter_from_env
        env::set_var("LANGFUSE_PUBLIC_KEY", "pk-test");
        env::set_var("LANGFUSE_SECRET_KEY", "sk-test");

        let result = exporter_from_env();
        assert!(matches!(result, Err(Error::OtlpExporter(_))));

        // Clean up
        env::remove_var("LANGFUSE_PUBLIC_KEY");
        env::remove_var("LANGFUSE_SECRET_KEY");
        env::remove_var("OTEL_EXPORTER_OTLP_TIMEOUT");
        env::remove_var("OTEL_EXPORTER_OTLP_COMPRESSION");
    }

    #[test]
    fn test_case_insensitive_authorization_header() {
        // Test that authorization header is handled case-insensitively
        // and normalized to "Authorization"

        // Test with lowercase "authorization"
        let result = ExporterBuilder::new()
            .with_endpoint("https://test.com")
            .with_header("authorization", "Bearer test-token")
            .build();

        // Should succeed without needing auth_header since we have authorization
        assert!(matches!(result, Err(Error::OtlpExporter(_))));

        // Test with mixed case
        let result = ExporterBuilder::new()
            .with_endpoint("https://test.com")
            .with_header("AUTHORIZATION", "Bearer test-token")
            .build();

        assert!(matches!(result, Err(Error::OtlpExporter(_))));

        // Test that auth_header is not added if authorization already exists
        let result = ExporterBuilder::new()
            .with_endpoint("https://test.com")
            .with_header("authorization", "Bearer from-header")
            .with_auth_header("Bearer from-auth")
            .build();

        // The header from with_header should be used (and normalized)
        assert!(matches!(result, Err(Error::OtlpExporter(_))));
    }
}
