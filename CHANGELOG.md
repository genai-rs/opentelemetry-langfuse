# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
- apply cargo fmt and update CLAUDE.md with CI check instructions
- remove test_endpoint_construction.rs example
- clarify error variant usage
- add references to Langfuse OpenTelemetry documentation
- replace abandoned dotenv with dotenvy
- [**breaking**] provide only Langfuse exporter, not full tracer
- add constants for environment variables
- Add CLAUDE.md with development instructions
