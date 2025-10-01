//! Langfuse OTLP exporter configuration.
//!
//! This module provides the [`ExporterBuilder`] for configuring an OpenTelemetry
//! OTLP exporter that sends traces to Langfuse.
//!
//! See the [Langfuse OpenTelemetry documentation](https://langfuse.com/integrations/native/opentelemetry)
//! for more details about the integration.

use crate::{auth, endpoint, Error, Result};
use opentelemetry_otlp::{SpanExporter, WithExportConfig, WithHttpConfig};
use std::collections::HashMap;
use std::time::Duration;

/// Builder for configuring a Langfuse OTLP exporter.
pub struct ExporterBuilder {
    endpoint: Option<String>,
    auth_header: Option<String>,
    timeout: Option<Duration>,
    additional_headers: HashMap<String, String>,
    http_client: Option<reqwest::Client>,
}

impl ExporterBuilder {
    /// Creates a new ExporterBuilder.
    pub fn new() -> Self {
        Self {
            endpoint: None,
            auth_header: None,
            timeout: None,
            additional_headers: HashMap::new(),
            http_client: None,
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

    /// Sets a custom HTTP client for the exporter.
    ///
    /// By default, a new reqwest::Client will be created. Use this method
    /// if you need custom configuration like proxy settings, custom certificates,
    /// or connection pooling.
    ///
    /// # Arguments
    ///
    /// * `client` - The HTTP client to use
    pub fn with_http_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = Some(client);
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

    /// Creates an ExporterBuilder from environment variables.
    ///
    /// This method reads Langfuse-specific variables:
    /// - `LANGFUSE_HOST`: The base URL of your Langfuse instance (defaults to <https://cloud.langfuse.com>)
    /// - `LANGFUSE_PUBLIC_KEY`: Your Langfuse public key (required)
    /// - `LANGFUSE_SECRET_KEY`: Your Langfuse secret key (required)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use opentelemetry_langfuse::ExporterBuilder;
    /// use std::time::Duration;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Load from environment, then customize
    /// let exporter = ExporterBuilder::from_env()?
    ///     .with_timeout(Duration::from_secs(30))
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_env() -> Result<Self> {
        // Get Langfuse endpoint (defaults to cloud if not set)
        let langfuse_endpoint = endpoint::build_otlp_endpoint_from_env()?;

        // Get Langfuse credentials
        let auth = auth::build_auth_header_from_env()?;

        Ok(Self {
            endpoint: Some(langfuse_endpoint),
            auth_header: Some(auth),
            timeout: None,
            additional_headers: HashMap::new(),
            http_client: None,
        })
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

        // Handle Authorization header with proper precedence:
        // 1. auth_header (from with_auth_header/with_basic_auth) takes precedence
        // 2. Otherwise use authorization from additional_headers (normalized)
        // 3. Error if neither is present

        if let Some(auth_header) = self.auth_header {
            // Remove any existing authorization headers (case-insensitive)
            // since auth_header takes precedence
            let auth_keys: Vec<String> = headers
                .keys()
                .filter(|k| k.eq_ignore_ascii_case("authorization"))
                .cloned()
                .collect();

            for key in auth_keys {
                headers.remove(&key);
            }

            // Insert the auth_header with normalized key
            headers.insert("Authorization".to_string(), auth_header);
        } else {
            // No explicit auth_header, check if we have one in additional_headers
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
                // No Authorization header found anywhere
                return Err(Error::MissingConfiguration(
                    "Authorization header or Langfuse credentials",
                ));
            }
        }

        // Build HTTP config with client
        let http_client = self.http_client.unwrap_or_default();

        let mut http_config = SpanExporter::builder()
            .with_http()
            .with_http_client(http_client)
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
    use std::env;

    #[test]
    #[serial]
    fn test_exporter_from_env_uses_langfuse_variables() {
        env::set_var("LANGFUSE_HOST", "https://test.langfuse.com");
        env::set_var("LANGFUSE_PUBLIC_KEY", "pk-test");
        env::set_var("LANGFUSE_SECRET_KEY", "sk-test");

        let result = ExporterBuilder::from_env().and_then(|b| b.build());
        assert!(result.is_ok());

        env::remove_var("LANGFUSE_HOST");
        env::remove_var("LANGFUSE_PUBLIC_KEY");
        env::remove_var("LANGFUSE_SECRET_KEY");
    }

    #[test]
    #[serial]
    fn test_exporter_from_env_missing_credentials() {
        env::set_var("LANGFUSE_HOST", "https://test.langfuse.com");

        let result = ExporterBuilder::from_env();
        assert!(matches!(result, Err(Error::MissingEnvironmentVariable(_))));

        env::remove_var("LANGFUSE_HOST");
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
        assert!(result.is_ok());

        // Test with mixed case
        let result = ExporterBuilder::new()
            .with_endpoint("https://test.com")
            .with_header("AUTHORIZATION", "Bearer test-token")
            .build();

        assert!(result.is_ok());

        // Test that auth_header takes precedence over header from with_header
        let result = ExporterBuilder::new()
            .with_endpoint("https://test.com")
            .with_header("authorization", "Bearer from-header")
            .with_auth_header("Bearer from-auth")
            .build();

        // The auth_header should take precedence over with_header
        // This will succeed with "Bearer from-auth"
        assert!(result.is_ok());

        // Test with basic_auth taking precedence
        let result = ExporterBuilder::new()
            .with_endpoint("https://test.com")
            .with_header("Authorization", "Bearer from-header")
            .with_basic_auth("user", "pass")
            .build();

        // The basic_auth should take precedence (creating "Basic dXNlcjpwYXNz")
        assert!(result.is_ok());
    }
}
