//! Langfuse OTLP exporter configuration.

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

    /// Sets the Langfuse credentials directly.
    ///
    /// # Arguments
    ///
    /// * `public_key` - The Langfuse public key
    /// * `secret_key` - The Langfuse secret key
    pub fn with_credentials(mut self, public_key: &str, secret_key: &str) -> Self {
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
        self.additional_headers.extend(
            headers
                .into_iter()
                .map(|(k, v)| (k.into(), v.into())),
        );
        self
    }

    /// Loads configuration from environment variables.
    ///
    /// This method reads:
    /// - LANGFUSE_HOST for the endpoint
    /// - LANGFUSE_PUBLIC_KEY and LANGFUSE_SECRET_KEY for authentication
    pub fn from_env(mut self) -> Result<Self> {
        self.endpoint = Some(endpoint::build_otlp_endpoint_from_env()?);
        self.auth_header = Some(auth::build_auth_header_from_env()?);
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
        let auth_header = self
            .auth_header
            .ok_or(Error::MissingConfiguration("auth_header"))?;

        // Create headers for authentication
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), auth_header);
        
        // Add any additional headers
        headers.extend(self.additional_headers);

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

/// Creates a Langfuse OTLP exporter using environment variables.
///
/// This is a convenience function that reads configuration from environment variables:
/// - LANGFUSE_HOST: The base URL of your Langfuse instance
/// - LANGFUSE_PUBLIC_KEY: Your Langfuse public key
/// - LANGFUSE_SECRET_KEY: Your Langfuse secret key
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
        .with_credentials(public_key, secret_key)
        .build()
}