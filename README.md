# opentelemetry-langfuse

[![Crates.io](https://img.shields.io/crates/v/opentelemetry-langfuse.svg)](https://crates.io/crates/opentelemetry-langfuse)
[![Documentation](https://docs.rs/opentelemetry-langfuse/badge.svg)](https://docs.rs/opentelemetry-langfuse)
[![CI](https://github.com/genai-rs/opentelemetry-langfuse/workflows/CI/badge.svg)](https://github.com/genai-rs/opentelemetry-langfuse/actions)
[![MSRV](https://img.shields.io/badge/MSRV-1.82-blue)](https://blog.rust-lang.org/2024/10/17/Rust-1.82.0.html)
[![License](https://img.shields.io/crates/l/opentelemetry-langfuse)](./LICENSE-MIT)

OpenTelemetry exporter for [Langfuse](https://langfuse.com), the open-source LLM observability platform.

This crate provides a configured OTLP exporter that sends OpenTelemetry traces to Langfuse. For more information about OpenTelemetry support in Langfuse, see the [official Langfuse OpenTelemetry documentation](https://langfuse.com/integrations/native/opentelemetry).

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

The exporter can be configured using environment variables. It supports both Langfuse-specific and standard OpenTelemetry variables:

### Langfuse Variables
```bash
LANGFUSE_PUBLIC_KEY=pk-lf-...              # Required: Your public key
LANGFUSE_SECRET_KEY=sk-lf-...              # Required: Your secret key
LANGFUSE_HOST=https://cloud.langfuse.com   # Optional: Defaults to cloud instance
```

### Standard OpenTelemetry Variables (Alternative)
```bash
OTEL_EXPORTER_OTLP_TRACES_ENDPOINT=https://cloud.langfuse.com/api/public/otel
OTEL_EXPORTER_OTLP_HEADERS="Authorization=Basic <base64_encoded_credentials>"
```

## Manual Configuration

You can also configure the exporter programmatically:

```rust
use opentelemetry_langfuse::ExporterBuilder;
use std::time::Duration;

let exporter = ExporterBuilder::new()
    .with_host("https://cloud.langfuse.com")
    .with_basic_auth("pk-lf-...", "sk-lf-...")  // or with_credentials()
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

## Resources

- [Langfuse OpenTelemetry Integration Guide](https://langfuse.com/integrations/native/opentelemetry)
- [Langfuse Documentation](https://langfuse.com/docs)
- [OpenTelemetry Rust](https://github.com/open-telemetry/opentelemetry-rust)
- [Langfuse API Reference](https://api.reference.langfuse.com)