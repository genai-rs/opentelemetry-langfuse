# opentelemetry-langfuse

[![Crates.io](https://img.shields.io/crates/v/opentelemetry-langfuse.svg)](https://crates.io/crates/opentelemetry-langfuse)
[![Documentation](https://docs.rs/opentelemetry-langfuse/badge.svg)](https://docs.rs/opentelemetry-langfuse)
[![CI](https://github.com/genai-rs/opentelemetry-langfuse/workflows/CI/badge.svg)](https://github.com/genai-rs/opentelemetry-langfuse/actions)
[![codecov](https://codecov.io/gh/genai-rs/opentelemetry-langfuse/branch/main/graph/badge.svg)](https://codecov.io/gh/genai-rs/opentelemetry-langfuse)
[![MSRV](https://img.shields.io/badge/MSRV-1.82-blue)](https://blog.rust-lang.org/2024/10/17/Rust-1.82.0.html)
[![License](https://img.shields.io/crates/l/opentelemetry-langfuse)](./LICENSE-MIT)

OpenTelemetry integration for [Langfuse](https://langfuse.com), the open-source LLM observability platform.

This crate provides OpenTelemetry components and utilities for integrating with Langfuse, enabling comprehensive observability for LLM applications. For more information about OpenTelemetry support in Langfuse, see the [official Langfuse OpenTelemetry documentation](https://langfuse.com/integrations/native/opentelemetry).

## Installation

```toml
[dependencies]
opentelemetry-langfuse = "*"
opentelemetry_sdk = { version = "0.31", features = [
    "trace",
    "rt-tokio",
    "experimental_trace_batch_span_processor_with_async_runtime"
]}
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

## Quick Start

```rust
use opentelemetry::global;
use opentelemetry::KeyValue;
use opentelemetry_langfuse::ExporterBuilder;
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessor;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the Langfuse exporter
    let exporter = ExporterBuilder::from_env()?.build()?;

    // Build tracer provider with BatchSpanProcessor
    // IMPORTANT: Explicitly provide Tokio runtime to avoid "no reactor running" panic
    let provider = SdkTracerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_attributes([KeyValue::new("service.name", "my-service")])
                .build(),
        )
        .with_span_processor(BatchSpanProcessor::builder(exporter, Tokio).build())
        .build();

    // Set as global provider
    global::set_tracer_provider(provider.clone());

    // Your application code here...

    // Shutdown to ensure all spans are flushed
    provider.shutdown()?;

    Ok(())
}
```

This uses `BatchSpanProcessor` with the async runtime for efficient batched exports. This is validated in our [integration tests](tests/integration_test.rs).

## Examples

The [`examples/`](examples) directory contains ready-to-run scenarios that map to common deployment patterns:

- [`sync_simple`](examples/sync_simple.rs) – minimal setup for development and quick demos
- [`sync_batch`](examples/sync_batch.rs) – synchronous app that spins up a lightweight async runtime for batching
- [`async_batch`](examples/async_batch.rs) – fully async application using Tokio; recommended for production loads
- [`custom_config`](examples/custom_config.rs) – advanced exporter configuration (custom HTTP clients, TLS, proxy, headers)

## Configuration

The exporter can be configured using Langfuse-specific environment variables:

```bash
LANGFUSE_PUBLIC_KEY=pk-lf-...              # Your public key (required)
LANGFUSE_SECRET_KEY=sk-lf-...              # Your secret key (required)
LANGFUSE_HOST=https://cloud.langfuse.com   # Optional: Defaults to cloud instance
```

Use `ExporterBuilder::from_env()` to load these variables:

```rust
use opentelemetry_langfuse::ExporterBuilder;
use std::time::Duration;

// Load from environment
let exporter = ExporterBuilder::from_env()?.build()?;

// Or load from environment and customize
let exporter = ExporterBuilder::from_env()?
    .with_timeout(Duration::from_secs(30))
    .build()?;
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

## Custom HTTP Client

By default, the OTLP exporter will use its own HTTP client with TLS support. You can provide a custom client for advanced configurations:
- Proxy settings
- Custom root certificates
- Connection pooling
- Custom timeout configurations

```rust
use opentelemetry_langfuse::ExporterBuilder;
use std::time::Duration;

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

## Context Helpers

Similar to the [langfuse-python SDK](https://langfuse.com/docs/sdk/python#update-trace), this crate provides a `LangfuseContext` struct for managing trace-level attributes:

```rust
use opentelemetry_langfuse::LangfuseContext;

// Create a context instance
let context = LangfuseContext::new();

// Use fluent API for configuration
context
    .set_session_id("session-123")
    .set_user_id("user-456")
    .add_tags(vec!["production".to_string(), "api-v2".to_string()])
    .set_metadata(serde_json::json!({
        "environment": "production",
        "version": "1.0.0"
    }));

// Get attributes to add to spans
let attributes = context.get_attributes();
```

**Design Note:** `LangfuseContext` uses an instance-based design (no global state). Create instances as needed for your use case.

For an example integration, see the [openai-ergonomic](https://github.com/genai-rs/openai-ergonomic) library's `LangfuseInterceptor`.

## Testing

The integration tests in [`tests/integration_test.rs`](tests/integration_test.rs) verify that traces are successfully exported to Langfuse and can be queried via the Langfuse API. The tests cover:

- **SimpleSpanProcessor**: Immediate (blocking) export
- **BatchSpanProcessor (async runtime)**: Batched export with Tokio integration

To run the integration tests:

```bash
export LANGFUSE_PUBLIC_KEY="pk-lf-..."
export LANGFUSE_SECRET_KEY="sk-lf-..."
export LANGFUSE_HOST="https://cloud.langfuse.com"

cargo test --test integration_test
```

The tests use unique timestamp-based IDs to track traces and verify they land in Langfuse by querying the API with the [`langfuse-ergonomic`](https://github.com/cstrnt/langfuse-ergonomic) client.

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
