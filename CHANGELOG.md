# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Default HTTP client creation to prevent `NoHttpClient` errors
- Comprehensive examples for different usage patterns:
  - `sync_simple` - SimpleSpanProcessor with immediate export
  - `sync_batch` - Batch processing in mostly synchronous applications
  - `async_batch` - Full async with Tokio and batch processing
  - `custom_config` - Advanced configuration with proxy, TLS, and headers

### Changed

- Exporter now provides a default `reqwest::Client` when none is specified
- Improved documentation to clarify async runtime requirements for HTTP exporters

## [0.3.1](https://github.com/genai-rs/opentelemetry-langfuse/compare/v0.3.0...v0.3.1) - 2025-08-31

### Fixed

- correct println output in examples and add whitespace trimming
- add /v1/traces to endpoint URL (fixes #11)

## [0.3.0](https://github.com/genai-rs/opentelemetry-langfuse/compare/v0.2.0...v0.3.0) - 2025-08-31

### Fixed

- add HTTP client configuration to fix NoHttpClient error
- remove invalid previous_version variable from git_release_body template

## [0.2.0](https://github.com/genai-rs/opentelemetry-langfuse/compare/v0.1.0...v0.2.0) - 2025-08-31

### Added

- add comprehensive OTEL environment variable support
- major improvements to OpenTelemetry-Langfuse integration
- implement suggested improvements
- add trace verification using langfuse-ergonomic
- Add OpenTelemetry tracer configuration for Langfuse

### Fixed

- address final review feedback
- auth_header should take precedence over headers from with_header
- address review feedback on endpoint examples and Authorization header handling
- ensure OTEL_EXPORTER_OTLP_TIMEOUT and COMPRESSION are considered in all env functions
- prioritize Langfuse env vars over OTEL vars
- address PR review feedback
- Add missing crate-level documentation

### Other

- Fix compression documentation consistency
- Fix rustdoc bare URLs warning
- Fix clippy linting issues
- Document that OTEL_EXPORTER_OTLP_COMPRESSION is not supported
- apply cargo fmt and update automation guide with CI check instructions
- remove test_endpoint_construction.rs example
- clarify error variant usage
- add references to Langfuse OpenTelemetry documentation
- replace abandoned dotenv with dotenvy
- [**breaking**] provide only Langfuse exporter, not full tracer
- add constants for environment variables
- Add automation guide with development instructions
