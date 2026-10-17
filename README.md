# opentelemetry-langfuse

[![Crates.io](https://img.shields.io/crates/v/opentelemetry-langfuse.svg)](https://crates.io/crates/opentelemetry-langfuse)
[![Documentation](https://docs.rs/opentelemetry-langfuse/badge.svg)](https://docs.rs/opentelemetry-langfuse)
[![CI](https://github.com/genai-rs/opentelemetry-langfuse/workflows/CI/badge.svg)](https://github.com/genai-rs/opentelemetry-langfuse/actions)
[![codecov](https://codecov.io/gh/genai-rs/opentelemetry-langfuse/branch/main/graph/badge.svg)](https://codecov.io/gh/genai-rs/opentelemetry-langfuse)
[![MSRV](https://img.shields.io/badge/MSRV-1.93.1-blue)](https://www.rust-lang.org/tools/install)
[![License](https://img.shields.io/crates/l/opentelemetry-langfuse)](./LICENSE-MIT)

OpenTelemetry integration for [Langfuse](https://langfuse.com), the open-source LLM observability platform.

This crate provides OpenTelemetry components and utilities for integrating with Langfuse, enabling comprehensive observability for LLM applications. For more information about OpenTelemetry support in Langfuse, see the [official Langfuse OpenTelemetry documentation](https://langfuse.com/integrations/native/opentelemetry).

## Installation

```toml
[dependencies]
opentelemetry-langfuse = "0.7.0"
opentelemetry = { version = "0.33.1", features = ["trace"] }
opentelemetry_sdk = { version = "0.33.1", features = [
    "trace",
    "rt-tokio",
    "experimental_trace_batch_span_processor_with_async_runtime"
]}
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

This combination requires **Rust 1.93.1 or later**. `ExporterBuilder::build()`
returns `opentelemetry_otlp::SpanExporter` from the 0.33 series, implementing
`opentelemetry_sdk::trace::SpanExporter` from the 0.33 series. Use matching
0.33 `opentelemetry` and `opentelemetry_sdk` dependencies when constructing a
provider or creating spans for this exporter.

### Using alongside openai-ergonomic

`openai-ergonomic` 0.6.0 uses OpenTelemetry 0.32 and declares Rust 1.88. It can
coexist in an application with this crate, but that application must meet this
crate's Rust 1.93.1 minimum. Cargo may then resolve both 0.32 and 0.33 SDKs.
Their exporter traits, providers, tracers, contexts, and spans are distinct Rust
types: do not pass this 0.33 exporter to a 0.32 provider, or pass a 0.33 provider
to an API that expects a 0.32 provider. Each OpenTelemetry version also has its
own global provider; setting the 0.33 global provider does not configure 0.32
instrumentation.

Use separate tracing pipelines with each library's matching SDK, or explicitly
align dependencies in a separately reviewed integration. This crate does not
bridge the two versions or require an openai-ergonomic upgrade. The
`LangfuseContext` helper example linked below illustrates attribute conventions;
it does not imply cross-version provider compatibility.

### Migrating from the 0.6 dependency stack

Use Rust 1.93.1 or later, OpenTelemetry 0.33, and reqwest 0.13. The exporter
returns an OpenTelemetry 0.33 `SpanExporter`, and `with_http_client` now accepts
a reqwest 0.13 `Client`; clients and SDK types from the earlier versions are
not interchangeable. Update these dependencies together in your application.
`Error::OpenTelemetry` wraps `opentelemetry_sdk::error::OTelSdkError`, replacing
the removed `TraceError` type.

The integration tests and examples use `langfuse-ergonomic` 0.7.0 with
`langfuse-client-base` 0.15.0. In that API, a trace name is `Option<String>`
rather than `Option<Option<String>>`. Langfuse remains a test/example dependency;
the exporter itself sends OTLP directly.

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

The [external consumer fixture](examples/external-consumer) has its own manifest
and builds this crate as a path dependency, without the library's dev-dependencies.
CI compiles and runs it on Rust 1.93.1. It starts a local fake OTLP endpoint,
exports one span, verifies receipt, and explicitly shuts down the provider and
runtime. No Langfuse service or credentials are needed:

```bash
cargo +1.93.1 check --manifest-path examples/external-consumer/Cargo.toml
cargo +1.93.1 run --manifest-path examples/external-consumer/Cargo.toml
cargo +1.93.1 tree --manifest-path examples/external-consumer/Cargo.toml -i opentelemetry_sdk
```

The fixture should resolve one 0.33 SDK. In an application using both libraries,
inspect `cargo tree -d` and `cargo tree -i opentelemetry_sdk@0.33.1` (adjust the
patch version to the resolved version) to see which dependency uses each SDK.

The remaining [`examples/`](examples) use a Langfuse service and credentials:

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

For real-time reads through Langfuse Cloud/v4's Observations API v2, add
`.with_header("x-langfuse-ingestion-version", "4")` before `.build()`.
Langfuse documents this header in its [Public API guide](https://langfuse.com/docs/api-and-data-platform/features/public-api).
Without it, direct OTLP exports can take up to 15 minutes to appear in v2 reads.

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

**Note on TLS**: The `opentelemetry-otlp` crate's `reqwest-client` feature supplies
the async HTTP transport, and `reqwest-rustls` explicitly enables HTTPS. This
also applies when building the library without its test dependencies. Custom
clients use reqwest 0.13 and can configure their own TLS backend.

The default HTTP client uses OTLP's resolved request timeout (10 seconds unless
overridden by OTLP environment variables or `with_timeout`). Custom clients must
set their own `reqwest::ClientBuilder::timeout`, as in the example above:
the exporter's retry budget cannot interrupt an unbounded custom-client request.

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

Unit tests and the local HTTP transport/query regression tests do not need credentials:

```bash
cargo test --lib --test otlp_transport_test --test langfuse_query_test
cargo test --doc
```

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

The live tests require Langfuse Cloud or self-hosted v4. They query the exact
exported trace ID, span ID, and name through Observations API v2, with a bounded
time range and a five-minute polling budget. They reuse the
[`langfuse-ergonomic`](https://github.com/genai-rs/langfuse-ergonomic) client's
authenticated transport, respect `Retry-After` on rate limits, and fail immediately
on authentication errors. Both ingestion and verification use `LANGFUSE_HOST`.

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
