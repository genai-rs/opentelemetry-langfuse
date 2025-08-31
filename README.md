# opentelemetry-langfuse

[![Crates.io](https://img.shields.io/crates/v/opentelemetry-langfuse.svg)](https://crates.io/crates/opentelemetry-langfuse)
[![Documentation](https://docs.rs/opentelemetry-langfuse/badge.svg)](https://docs.rs/opentelemetry-langfuse)
[![CI](https://github.com/genai-rs/opentelemetry-langfuse/workflows/CI/badge.svg)](https://github.com/genai-rs/opentelemetry-langfuse/actions)
[![MSRV](https://img.shields.io/badge/MSRV-1.82-blue)](https://blog.rust-lang.org/2024/10/17/Rust-1.82.0.html)
[![License](https://img.shields.io/crates/l/opentelemetry-langfuse)](./LICENSE-MIT)

OpenTelemetry integration for [Langfuse](https://langfuse.com), the open-source LLM observability platform.

This crate provides OpenTelemetry components and utilities for integrating with Langfuse, enabling comprehensive observability for LLM applications. For more information about OpenTelemetry support in Langfuse, see the [official Langfuse OpenTelemetry documentation](https://langfuse.com/integrations/native/opentelemetry).

## Features

- 🚀 **OTLP Exporter** - Configured exporter for sending traces to Langfuse
- 🔌 **Composable** - Integrates with your existing OpenTelemetry setup
- 🏗️ **Builder Pattern** - Flexible configuration API
- 🔐 **Secure** - Handles authentication with Langfuse credentials
- 🌐 **Dual Configuration** - Supports both Langfuse-specific and standard OTEL environment variables

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

## Configuration

The exporter can be configured using environment variables. You have three options:

### Option 1: Langfuse-Specific Variables
Use `exporter_from_langfuse_env()` with these variables:
```bash
LANGFUSE_PUBLIC_KEY=pk-lf-...              # Your public key
LANGFUSE_SECRET_KEY=sk-lf-...              # Your secret key
LANGFUSE_HOST=https://cloud.langfuse.com   # Optional: Defaults to cloud instance
```

### Option 2: Standard OpenTelemetry Variables
Use `exporter_from_otel_env()` following the [OTLP Exporter specification](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/protocol/exporter.md#endpoint-urls-for-otlphttp):
```bash
# For endpoint (use ONE of these):
# Option A: Direct traces endpoint (recommended)
OTEL_EXPORTER_OTLP_TRACES_ENDPOINT=https://cloud.langfuse.com/api/public/otel

# Option B: Base endpoint that works with Langfuse
OTEL_EXPORTER_OTLP_ENDPOINT=https://cloud.langfuse.com/api/public/otel  # /v1/traces will be appended
# This creates: https://cloud.langfuse.com/api/public/otel/v1/traces (which Langfuse accepts)

# For authentication
OTEL_EXPORTER_OTLP_HEADERS="Authorization=Basic <base64_encoded_credentials>"
```

⚠️ **Important**: Do NOT use `OTEL_EXPORTER_OTLP_ENDPOINT=https://cloud.langfuse.com/api/public` as this would create `/api/public/v1/traces` which Langfuse does not accept.

### Option 3: Automatic Fallback
Use `exporter_from_env()` for automatic fallback with sensible defaults. Priority order:

**For endpoint:**
1. `LANGFUSE_HOST` (appends `/api/public/otel`)
2. `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT`
3. `OTEL_EXPORTER_OTLP_ENDPOINT` (appends `/v1/traces`)
4. **Default**: `https://cloud.langfuse.com/api/public/otel` (when no endpoint variables are set)

**For authentication:**
1. `LANGFUSE_PUBLIC_KEY` + `LANGFUSE_SECRET_KEY`
2. `OTEL_EXPORTER_OTLP_TRACES_HEADERS`
3. `OTEL_EXPORTER_OTLP_HEADERS`

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