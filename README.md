# opentelemetry-langfuse

[![Crates.io](https://img.shields.io/crates/v/opentelemetry-langfuse.svg)](https://crates.io/crates/opentelemetry-langfuse)
[![Documentation](https://docs.rs/opentelemetry-langfuse/badge.svg)](https://docs.rs/opentelemetry-langfuse)
[![CI](https://github.com/genai-rs/opentelemetry-langfuse/workflows/CI/badge.svg)](https://github.com/genai-rs/opentelemetry-langfuse/actions)
[![MSRV](https://img.shields.io/badge/MSRV-1.82-blue)](https://blog.rust-lang.org/2024/10/17/Rust-1.82.0.html)
[![License](https://img.shields.io/crates/l/opentelemetry-langfuse)](./LICENSE-MIT)

OpenTelemetry integration for [Langfuse](https://langfuse.com), the open-source LLM observability platform.

This crate provides OpenTelemetry components and utilities for integrating with Langfuse, enabling comprehensive observability for LLM applications. For more information about OpenTelemetry support in Langfuse, see the [official Langfuse OpenTelemetry documentation](https://langfuse.com/integrations/native/opentelemetry).

## Installation

```toml
[dependencies]
opentelemetry-langfuse = "*"
```

## Quick Start

```rust
use opentelemetry::global;
use opentelemetry_langfuse::exporter_from_env;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;
use opentelemetry::KeyValue;

// Create the Langfuse exporter
let exporter = exporter_from_env()?;

// Build your tracer provider with the exporter
let provider = SdkTracerProvider::builder()
    .with_batch_exporter(exporter)
    .with_resource(Resource::builder().with_attributes(vec![
        KeyValue::new("service.name", "my-service"),
    ]).build())
    .build();

// Set as global provider and start tracing
global::set_tracer_provider(provider);
```

## Examples

The [`examples/`](examples) directory contains ready-to-run scenarios that map to common deployment patterns:

- [`sync_simple`](examples/sync_simple.rs) – minimal setup for development and quick demos
- [`sync_batch`](examples/sync_batch.rs) – synchronous app that spins up a lightweight async runtime for batching
- [`async_batch`](examples/async_batch.rs) – fully async application using Tokio; recommended for production loads
- [`custom_config`](examples/custom_config.rs) – advanced exporter configuration (custom HTTP clients, TLS, proxy, headers)

Run any example with:
```bash
export LANGFUSE_PUBLIC_KEY="pk-lf-..."
export LANGFUSE_SECRET_KEY="sk-lf-..."
export LANGFUSE_HOST="https://cloud.langfuse.com"

cargo run --example <example_name>
```

## Configuration

The exporter can be configured using Langfuse-specific environment variables:

```bash
LANGFUSE_PUBLIC_KEY=pk-lf-...              # Your public key (required)
LANGFUSE_SECRET_KEY=sk-lf-...              # Your secret key (required)
LANGFUSE_HOST=https://cloud.langfuse.com   # Optional: Defaults to cloud instance
```

Use `exporter_from_env()` to load these variables:

```rust
use opentelemetry_langfuse::exporter_from_env;

let exporter = exporter_from_env()?;
```

## Manual Configuration

You can also configure the exporter programmatically:

```rust
use opentelemetry_langfuse::ExporterBuilder;
use std::time::Duration;

let exporter = ExporterBuilder::new()
    .with_host("https://cloud.langfuse.com")
    .with_basic_auth("pk-lf-...", "sk-lf-...")
    .with_timeout(Duration::from_secs(10))
    .build()?;
```

## Async Runtime Considerations

This crate does not directly specify async runtime features, allowing flexibility in how you configure your async runtime.

**Important:** HTTP exporters require an async runtime for network operations (the underlying HTTP client is `reqwest`, which depends on Tokio). The `opentelemetry-otlp` crate's `reqwest-client` feature brings in Tokio as a transitive dependency.

When using `BatchSpanProcessor` (recommended for production), the async runtime is also needed for the batching mechanism itself. For applications without an existing async runtime, you'll need to create one - see the `sync_batch` example for how to do this with minimal overhead.

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use opentelemetry_langfuse::exporter_from_env;
    use opentelemetry_sdk::trace::SdkTracerProvider;

    // Create the exporter
    let exporter = exporter_from_env()?;

    // Use with batch processing (recommended for production)
    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .build();

    // The batch processor will handle spans asynchronously
    Ok(())
}
```

For end-to-end samples, see the `async_batch`, `sync_batch`, and `sync_simple` examples described above.

### Custom HTTP Client

By default, the OTLP exporter will use its own HTTP client with TLS support. You can provide a custom client for advanced configurations:
- Proxy settings
- Custom root certificates
- Connection pooling
- Custom timeout configurations

```rust
use opentelemetry_langfuse::ExporterBuilder;
use std::time::Duration;

// Note: reqwest version should match what opentelemetry-otlp uses (0.12)
let custom_client = reqwest::Client::builder()
    .timeout(Duration::from_secs(30))
    .proxy(reqwest::Proxy::http("http://proxy.example.com:8080")?)
    .build()?;

let exporter = ExporterBuilder::new()
    .with_host("https://cloud.langfuse.com")
    .with_basic_auth("pk-lf-...", "sk-lf-...")
    .with_http_client(custom_client)
    .build()?;
```

**Note on TLS**: TLS support comes from the `opentelemetry-otlp` crate's `reqwest-client` feature. If you're building a custom client with specific TLS requirements, ensure your `reqwest` client is configured with appropriate TLS features.


## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Resources

- [Langfuse OpenTelemetry Integration Guide](https://langfuse.com/integrations/native/opentelemetry)
- [Langfuse Documentation](https://langfuse.com/docs)
- [OpenTelemetry Rust](https://github.com/open-telemetry/opentelemetry-rust)
- [Langfuse API Reference](https://api.reference.langfuse.com)
