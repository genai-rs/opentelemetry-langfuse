# opentelemetry-langfuse

[![Crates.io](https://img.shields.io/crates/v/opentelemetry-langfuse.svg)](https://crates.io/crates/opentelemetry-langfuse)
[![Documentation](https://docs.rs/opentelemetry-langfuse/badge.svg)](https://docs.rs/opentelemetry-langfuse)
[![CI](https://github.com/genai-rs/opentelemetry-langfuse/workflows/CI/badge.svg)](https://github.com/genai-rs/opentelemetry-langfuse/actions)
[![MSRV](https://img.shields.io/badge/MSRV-1.82-blue)](https://blog.rust-lang.org/2024/10/17/Rust-1.82.0.html)
[![License](https://img.shields.io/crates/l/opentelemetry-langfuse)](./LICENSE-MIT)

OpenTelemetry integration for [Langfuse](https://langfuse.com), the open-source LLM observability platform.

## Features

🚧 **Under Development** - This library is currently being built

- 🔭 **OpenTelemetry Native** - Built on OpenTelemetry standards
- 🔄 **Seamless Integration** - Export traces directly to Langfuse
- 🏗️ **Builder Pattern** - Intuitive API design
- 🔒 **Type Safe** - Strongly typed with compile-time guarantees
- 📊 **Comprehensive** - Support for traces, spans, and metrics
- ⚡ **Production Ready** - Built for reliability and performance

## Installation

```toml
[dependencies]
opentelemetry-langfuse = "*"
```

## Quick Start

Coming soon...

## Configuration

Set these environment variables:

```bash
LANGFUSE_PUBLIC_KEY=pk-lf-...
LANGFUSE_SECRET_KEY=sk-lf-...
LANGFUSE_HOST=https://cloud.langfuse.com  # Optional, defaults to cloud instance
```

## Examples

Examples will be added as the library develops.

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