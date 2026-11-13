# opentelemetry-langfuse

[![Crates.io](https://img.shields.io/crates/v/opentelemetry-langfuse)](https://crates.io/crates/opentelemetry-langfuse)
[![docs.rs](https://img.shields.io/docsrs/opentelemetry-langfuse)](https://docs.rs/opentelemetry-langfuse)
[![MSRV](https://img.shields.io/crates/msrv/opentelemetry-langfuse)](Cargo.toml)
[![Dependency status](https://deps.rs/crate/opentelemetry-langfuse/latest/status.svg)](https://deps.rs/crate/opentelemetry-langfuse)
[![CI](https://github.com/genai-rs/opentelemetry-langfuse/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/genai-rs/opentelemetry-langfuse/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/genai-rs/opentelemetry-langfuse/branch/main/graph/badge.svg)](https://codecov.io/gh/genai-rs/opentelemetry-langfuse)
[![License](https://img.shields.io/crates/l/opentelemetry-langfuse)](#license)

OpenTelemetry integration for [Langfuse](https://langfuse.com), the open-source
LLM observability platform: an OTLP/HTTP span exporter preconfigured for
Langfuse (endpoint, authentication, environment variables) and helpers for
Langfuse trace attributes. See Langfuse's
[OpenTelemetry documentation](https://langfuse.com/integrations/native/opentelemetry)
for what Langfuse does with the spans.

## Quick start

```bash
cargo add opentelemetry-langfuse
cargo add opentelemetry --features trace
cargo add opentelemetry_sdk --features trace,rt-tokio,experimental_trace_batch_span_processor_with_async_runtime
cargo add tokio --features rt-multi-thread,macros
```

Your `opentelemetry` and `opentelemetry_sdk` must come from the same **0.33**
OpenTelemetry family as this crate ([compatibility](docs/compatibility.md)).

```rust,no_run
use opentelemetry::{global, KeyValue};
use opentelemetry_langfuse::ExporterBuilder;
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessor;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Reads LANGFUSE_PUBLIC_KEY, LANGFUSE_SECRET_KEY and LANGFUSE_HOST
    let exporter = ExporterBuilder::from_env()?.build()?;

    let provider = SdkTracerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_attributes([KeyValue::new("service.name", "my-service")])
                .build(),
        )
        .with_span_processor(BatchSpanProcessor::builder(exporter, Tokio).build())
        .build();
    global::set_tracer_provider(provider.clone());

    // Your application code here...

    provider.shutdown()?; // flushes the remaining spans
    Ok(())
}
```

More in [getting started](docs/getting-started.md).

## Features

- **Langfuse exporter**: `ExporterBuilder` builds an `opentelemetry_otlp::SpanExporter`
  with the Langfuse OTLP endpoint and Basic authentication, from
  [environment variables or code](docs/configuration.md).
- **HTTP client control**: timeouts, extra headers, and a
  [custom `reqwest` client](docs/configuration.md#custom-http-client) for proxies
  and certificates.
- **Trace attributes**: [`LangfuseContext`](docs/context.md) sets Langfuse's
  session, user, tags and metadata attributes.
- **Tested**: unit and local transport tests, an
  [external consumer](docs/development.md#external-consumer) built at the MSRV,
  and live integration tests against Langfuse.

## Documentation

| Page | Contents |
| ---- | -------- |
| [Getting started](docs/getting-started.md) | Installation, Cargo features, a first exporter, examples |
| [Configuration](docs/configuration.md) | Environment variables, builder options, ingestion version header, custom HTTP client and TLS |
| [Trace attributes](docs/context.md) | `LangfuseContext` |
| [Compatibility](docs/compatibility.md) | The OpenTelemetry family requirement, MSRV, using alongside libraries on another OpenTelemetry version |
| [Migration](docs/migration.md) | Upgrading from the 0.6 dependency stack |
| [Development](docs/development.md) | Tests, the external consumer, local CI |
| [API reference](https://docs.rs/opentelemetry-langfuse) | docs.rs |
| [CHANGELOG](CHANGELOG.md) · [CONTRIBUTING](CONTRIBUTING.md) · [SECURITY](SECURITY.md) | |

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))
