# Migration

The [CHANGELOG](../CHANGELOG.md) lists every change; this page covers upgrades
that need changes in your application.

## From the 0.6 dependency stack

Use Rust 1.93.1 or later, OpenTelemetry 0.33, and reqwest 0.13. The exporter
returns an OpenTelemetry 0.33 `SpanExporter`, and `with_http_client` now accepts
a reqwest 0.13 `Client`; clients and SDK types from the earlier versions are
not interchangeable. Update these dependencies together in your application.
`Error::OpenTelemetry` wraps `opentelemetry_sdk::error::OTelSdkError`, replacing
the removed `TraceError` type.

As of that release, the integration tests and examples moved to
`langfuse-ergonomic` 0.7.0 with `langfuse-client-base` 0.15.0. In that API, a
trace name is `Option<String>` rather than `Option<Option<String>>`. Langfuse
remains a test/example dependency; the exporter itself sends OTLP directly.
