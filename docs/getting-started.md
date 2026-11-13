# Getting started

## Installation

```bash
cargo add opentelemetry-langfuse
cargo add opentelemetry --features trace
cargo add opentelemetry_sdk --features trace,rt-tokio,experimental_trace_batch_span_processor_with_async_runtime
cargo add tokio --features rt-multi-thread,macros
```

Or add `opentelemetry-langfuse = "0.x"` to `Cargo.toml` by hand, using the
version shown on the [crates.io](https://crates.io/crates/opentelemetry-langfuse)
badge.

**OpenTelemetry versions must match.** Your `opentelemetry` and
`opentelemetry_sdk` dependencies must come from the same **0.33** OpenTelemetry
family as this crate. See [compatibility](compatibility.md).

### Cargo features

- `rt-tokio` (default): enables `opentelemetry_sdk`'s `rt-tokio` and async-runtime
  `BatchSpanProcessor` support. Use `default-features = false` if you bring your
  own span processor and do not want the Tokio runtime pulled in.

## A first exporter

Set your Langfuse credentials ([configuration](configuration.md)):

```bash
export LANGFUSE_PUBLIC_KEY=pk-lf-...
export LANGFUSE_SECRET_KEY=sk-lf-...
export LANGFUSE_HOST=https://cloud.langfuse.com   # optional, the default
```

Then build the exporter and a tracer provider:

```rust,no_run
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

This uses `BatchSpanProcessor` with the async runtime for efficient batched
exports. It is validated in the [integration tests](../tests/integration_test.rs).

## Examples

The [`examples/`](../examples) use a Langfuse service and credentials:

- [`sync_simple`](../examples/sync_simple.rs) – minimal setup for development and quick demos
- [`sync_batch`](../examples/sync_batch.rs) – synchronous app that spins up a lightweight async runtime for batching
- [`async_batch`](../examples/async_batch.rs) – fully async application using Tokio; recommended for production loads
- [`custom_config`](../examples/custom_config.rs) – advanced exporter configuration (custom HTTP clients, TLS, proxy, headers)

```bash
cargo run --example async_batch
```

The [external consumer](development.md#external-consumer) needs no Langfuse
service: it exports to a local fake endpoint.
