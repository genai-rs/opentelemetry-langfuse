//! Authentication utilities for Langfuse.

use base64::{engine::general_purpose::STANDARD, Engine};
use std::env;

/// Builds a Langfuse authentication header value from public and secret keys.
///
/// This function concatenates the public and secret keys with a colon separator,
/// encodes them in base64, and returns the complete "Basic {auth}" string.
///
/// # Arguments
///
/// * `public_key` - The Langfuse public key
/// * `secret_key` - The Langfuse secret key
///
/// # Returns
///
/// Returns the complete authentication header value "Basic {base64_encoded}".
///
/// # Example
///
/// ```
/// use opentelemetry_langfuse::auth::build_auth_header;
///
/// let auth = build_auth_header(
///     "pk-lf-1234567890",
///     "sk-lf-1234567890"
/// );
/// assert!(auth.starts_with("Basic "));
/// ```
pub fn build_auth_header(public_key: &str, secret_key: &str) -> String {
    let auth_string = format!("{}:{}", public_key, secret_key);
    let encoded = STANDARD.encode(auth_string.as_bytes());
    format!("Basic {}", encoded)
}

/// Builds a Langfuse authentication header value from environment variables.
///
/// This function reads the LANGFUSE_PUBLIC_KEY and LANGFUSE_SECRET_KEY environment
/// variables and creates the complete authentication header value.
///
/// # Returns
///
/// Returns a Result containing the complete authentication header value "Basic {base64_encoded}",
/// or an error if environment variables are missing.
///
/// # Errors
///
/// Returns an error if either LANGFUSE_PUBLIC_KEY or LANGFUSE_SECRET_KEY
/// environment variables are not set.
///
/// # Example
///
/// ```no_run
/// use opentelemetry_langfuse::auth::build_auth_header_from_env;
///
/// // Requires LANGFUSE_PUBLIC_KEY and LANGFUSE_SECRET_KEY env vars
/// let auth = build_auth_header_from_env().unwrap();
/// ```
pub fn build_auth_header_from_env() -> Result<String, crate::Error> {
    let public_key = env::var("LANGFUSE_PUBLIC_KEY")
        .map_err(|_| crate::Error::MissingEnvironmentVariable("LANGFUSE_PUBLIC_KEY"))?;

    let secret_key = env::var("LANGFUSE_SECRET_KEY")
        .map_err(|_| crate::Error::MissingEnvironmentVariable("LANGFUSE_SECRET_KEY"))?;

    Ok(build_auth_header(&public_key, &secret_key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_auth_header() {
        let auth = build_auth_header("pk-lf-test", "sk-lf-secret");
        let expected = format!("Basic {}", STANDARD.encode("pk-lf-test:sk-lf-secret"));
        assert_eq!(auth, expected);
    }

    #[test]
    fn test_build_auth_header_from_env() {
        env::set_var("LANGFUSE_PUBLIC_KEY", "pk-env-test");
        env::set_var("LANGFUSE_SECRET_KEY", "sk-env-secret");

        let auth = build_auth_header_from_env().unwrap();
        let expected = format!("Basic {}", STANDARD.encode("pk-env-test:sk-env-secret"));
        assert_eq!(auth, expected);
    }

    #[test]
    fn test_missing_env_keys() {
        env::remove_var("LANGFUSE_PUBLIC_KEY");
        env::remove_var("LANGFUSE_SECRET_KEY");

        let result = build_auth_header_from_env();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            crate::Error::MissingEnvironmentVariable("LANGFUSE_PUBLIC_KEY")
        ));
    }
}
