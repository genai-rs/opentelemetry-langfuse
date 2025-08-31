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

    /// Sets the Basic authentication credentials.
    ///
    /// This is an alias for `with_credentials` that better matches HTTP terminology.
    ///
    /// # Arguments
    ///
    /// * `username` - The username (Langfuse public key)
    /// * `password` - The password (Langfuse secret key)
    pub fn with_basic_auth(self, username: &str, password: &str) -> Self {
        self.with_credentials(username, password)
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
    pub fn from_env(mut self) -> Result<Self> {
        // Check for Langfuse-specific endpoint first (may use default)
        let langfuse_endpoint = endpoint::build_otlp_endpoint_from_env()?;
        
        // Only use OTEL endpoints if LANGFUSE_HOST was not explicitly set
        if env::var(ENV_LANGFUSE_HOST).is_err() {
            // No LANGFUSE_HOST set, check for OTEL endpoints
            if let Ok(endpoint) = env::var(ENV_OTEL_EXPORTER_OTLP_TRACES_ENDPOINT) {
                self.endpoint = Some(endpoint);
            } else if let Ok(endpoint) = env::var(ENV_OTEL_EXPORTER_OTLP_ENDPOINT) {
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
            if let Ok(headers) = env::var(ENV_OTEL_EXPORTER_OTLP_TRACES_HEADERS)
                .or_else(|_| env::var(ENV_OTEL_EXPORTER_OTLP_HEADERS))
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
        
        // If no Authorization header yet, add the auth_header
        if !headers.contains_key("Authorization") {
            if let Some(auth_header) = self.auth_header {
                headers.insert("Authorization".to_string(), auth_header);
            } else {
                return Err(Error::MissingConfiguration("Authorization header or Langfuse credentials"));
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

/// Creates a Langfuse OTLP exporter using environment variables.
///
/// This is a convenience function that reads configuration from environment variables.
///
/// Langfuse-specific variables take precedence over standard OpenTelemetry variables:
///
/// For endpoint (in order of precedence):
/// - LANGFUSE_HOST: The base URL of your Langfuse instance
/// - OTEL_EXPORTER_OTLP_TRACES_ENDPOINT: Direct OTLP endpoint URL
/// - OTEL_EXPORTER_OTLP_ENDPOINT: Base OTLP endpoint (will append /v1/traces)
/// - Default: https://cloud.langfuse.com
///
/// For authentication (in order of precedence):
/// - LANGFUSE_PUBLIC_KEY + LANGFUSE_SECRET_KEY: Your Langfuse credentials
/// - OTEL_EXPORTER_OTLP_TRACES_HEADERS: Headers including Authorization
/// - OTEL_EXPORTER_OTLP_HEADERS: Headers including Authorization
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