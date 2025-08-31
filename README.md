# opentelemetry-langfuse

[![Crates.io](https://img.shields.io/crates/v/opentelemetry-langfuse.svg)](https://crates.io/crates/opentelemetry-langfuse)
[![Documentation](https://docs.rs/opentelemetry-langfuse/badge.svg)](https://docs.rs/opentelemetry-langfuse)
[![CI](https://github.com/genai-rs/opentelemetry-langfuse/workflows/CI/badge.svg)](https://github.com/genai-rs/opentelemetry-langfuse/actions)
[![MSRV](https://img.shields.io/badge/MSRV-1.82-blue)](https://blog.rust-lang.org/2024/10/17/Rust-1.82.0.html)
[![License](https://img.shields.io/crates/l/opentelemetry-langfuse)](./LICENSE-MIT)

OpenTelemetry exporter for [Langfuse](https://langfuse.com), the open-source LLM observability platform.

## Features

- 🎯 **Focused** - Provides a configured OTLP exporter specifically for Langfuse
- 🔌 **Composable** - Integrates with your existing OpenTelemetry setup
- 🏗️ **Builder Pattern** - Flexible configuration API
- 🔐 **Secure** - Handles authentication with Langfuse credentials
- 📦 **Lightweight** - Minimal dependencies, does one thing well

## Installation

```toml
[dependencies]
opentelemetry-langfuse = "*"
```

## Quick Start

```rust
use opentelemetry::global;
use opentelemetry_langfuse::exporter_from_env;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::Resource;
use opentelemetry::KeyValue;

// Create the Langfuse exporter
let exporter = exporter_from_env()?;

// Build your tracer provider with the exporter
let provider = TracerProvider::builder()
    .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
    .with_resource(Resource::new(vec![
        KeyValue::new("service.name", "my-service"),
    ]))
    .build();

// Set as global provider and start tracing
global::set_tracer_provider(provider);
```

## Configuration

Set these environment variables:

```bash
LANGFUSE_PUBLIC_KEY=pk-lf-...
LANGFUSE_SECRET_KEY=sk-lf-...
LANGFUSE_HOST=https://cloud.langfuse.com  # Optional, defaults to cloud instance
```

## Manual Configuration

You can also configure the exporter programmatically:

```rust
use opentelemetry_langfuse::ExporterBuilder;
use std::time::Duration;

let exporter = ExporterBuilder::new()
    .with_host("https://cloud.langfuse.com")
    .with_credentials("pk-lf-...", "sk-lf-...")
    .with_timeout(Duration::from_secs(10))
    .build()?;
```

## Examples

See the [examples](./examples) directory for complete working examples:

- [`basic.rs`](./examples/basic.rs) - Simple usage with environment variables
- [`manual_config.rs`](./examples/manual_config.rs) - Manual configuration without env vars

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Links

- [Langfuse Documentation](https://langfuse.com/docs)
- [OpenTelemetry Rust](https://github.com/open-telemetry/opentelemetry-rust)
- [API Reference](https://api.reference.langfuse.com)