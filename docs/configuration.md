# Configuration

## Environment variables

The exporter can be configured using Langfuse-specific environment variables:

```bash
LANGFUSE_PUBLIC_KEY=pk-lf-...              # Your public key (required)
LANGFUSE_SECRET_KEY=sk-lf-...              # Your secret key (required)
LANGFUSE_HOST=https://cloud.langfuse.com   # Optional: Defaults to cloud instance
```

Use `ExporterBuilder::from_env()` to load these variables:

```rust,no_run
use opentelemetry_langfuse::ExporterBuilder;
use std::time::Duration;

fn build() -> Result<(), Box<dyn std::error::Error>> {
    // Load from environment
    let exporter = ExporterBuilder::from_env()?.build()?;

    // Or load from environment and customize
    let exporter = ExporterBuilder::from_env()?
        .with_timeout(Duration::from_secs(30))
        .build()?;
    Ok(())
}
```

## Manual configuration

You can also configure the exporter programmatically:

```rust,no_run
use opentelemetry_langfuse::ExporterBuilder;
use std::time::Duration;

fn build() -> Result<(), Box<dyn std::error::Error>> {
    let exporter = ExporterBuilder::new()
        .with_host("https://cloud.langfuse.com")
        .with_basic_auth("pk-lf-...", "sk-lf-...")
        .with_timeout(Duration::from_secs(10))
        .build()?;
    Ok(())
}
```

`with_header` and `with_headers` add HTTP headers to every export.

## Real-time reads (ingestion version 4)

For real-time reads through Langfuse Cloud/v4's Observations API v2, add
`.with_header("x-langfuse-ingestion-version", "4")` before `.build()`.
Langfuse documents this header in its [Public API guide](https://langfuse.com/docs/api-and-data-platform/features/public-api).
Without it, direct OTLP exports can take up to 15 minutes to appear in v2 reads.

## Custom HTTP client

By default, the OTLP exporter will use its own HTTP client with TLS support. You can provide a custom client for advanced configurations:
- Proxy settings
- Custom root certificates
- Connection pooling
- Custom timeout configurations

```rust,no_run
use opentelemetry_langfuse::ExporterBuilder;
use std::time::Duration;

fn build() -> Result<(), Box<dyn std::error::Error>> {
    let custom_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .proxy(reqwest::Proxy::http("http://proxy.example.com:8080")?)
        .build()?;

    let exporter = ExporterBuilder::new()
        .with_host("https://cloud.langfuse.com")
        .with_basic_auth("pk-lf-...", "sk-lf-...")
        .with_http_client(custom_client)
        .build()?;
    Ok(())
}
```

**TLS.** The `opentelemetry-otlp` crate's `reqwest-client` feature supplies
the async HTTP transport, and `reqwest-rustls` explicitly enables HTTPS. This
also applies when building the library without its test dependencies. Custom
clients must come from the same `reqwest` minor release line this crate depends
on (see [`Cargo.toml`](../Cargo.toml)) and can configure their own TLS backend.

**Timeouts.** The default HTTP client uses OTLP's resolved request timeout (10
seconds unless overridden by OTLP environment variables or `with_timeout`).
Custom clients must set their own `reqwest::ClientBuilder::timeout`, as in the
example above: the exporter's retry budget cannot interrupt an unbounded
custom-client request.

See the [`custom_config`](../examples/custom_config.rs) example for more.
