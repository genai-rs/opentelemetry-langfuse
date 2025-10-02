# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.1](https://github.com/genai-rs/opentelemetry-langfuse/compare/v0.4.0...v0.4.1) - 2025-10-02

### Other

- updated documentation

## [0.4.0](https://github.com/genai-rs/opentelemetry-langfuse/compare/v0.3.1...v0.4.0) - 2025-10-01

### Added

- working integration test with SimpleSpanProcessor
- add working integration test with SimpleSpanProcessor
- add experimental_trace_batch_span_processor_with_async_runtime feature
- add integration tests for Langfuse exporter
- add default HTTP client and comprehensive examples

### Fixed

- use real Langfuse secrets in coverage workflow
- add Langfuse env vars to no-default-features test step
- use LANGFUSE_HOST from secrets instead of hardcoding
- use async runtime BatchSpanProcessor in tests
- avoid global tracer provider in tests to prevent hanging

### Other

- ensure coverage never blocks pipeline
- consolidate coverage into main CI workflow
- apply cargo fmt formatting
- simplify README to focus on recommended production approach
- [**breaking**] make ExporterBuilder::from_env() a static constructor
- prominently feature async runtime BatchSpanProcessor
- document that BatchSpanProcessor fails in both sync and async test contexts
- fmt
- document BatchSpanProcessor runtime limitation in tests
- add rt-tokio feature to enable async batch processor
- explicitly set LANGFUSE_HOST in test environment
- added rust code sample
- cleanup unused items
- remove items from readme
- remove outdated dual configuration feature from README
- remove unused tracing-subscriber dev-dependency
- remove support for standard OpenTelemetry environment variables
- remove CONTRIBUTING.md and AUTOMATION_GUIDE.md
- remove unused code and improve documentation accuracy
- revert CHANGELOG.md changes (managed by release-plz)
- add E2E verification to async runtime examples
- convert Claude guide references to automation

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
