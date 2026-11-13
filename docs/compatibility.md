# Compatibility

## OpenTelemetry versions must match

This crate is built against the OpenTelemetry **0.33** family
(`opentelemetry`, `opentelemetry_sdk`, `opentelemetry-otlp`). Your own
`opentelemetry` and `opentelemetry_sdk` dependencies must come from that same
minor release line; types from different OpenTelemetry minors are not
interchangeable. See this crate's [`Cargo.toml`](../Cargo.toml) for the exact
requirements, and expect the required family to move forward in future
releases (the [CHANGELOG](../CHANGELOG.md) notes each bump).

`ExporterBuilder::build()` returns an `opentelemetry_otlp::SpanExporter`
implementing `opentelemetry_sdk::trace::SpanExporter` from the same
OpenTelemetry family this crate depends on. Use matching `opentelemetry` and
`opentelemetry_sdk` dependencies when constructing a provider or creating spans
for this exporter.

The same holds for `reqwest`: a client passed to `with_http_client` must come
from the `reqwest` minor release line in [`Cargo.toml`](../Cargo.toml)
([custom HTTP client](configuration.md#custom-http-client)).

## Rust version

The minimum supported Rust version is shown on the MSRV badge (the
`rust-version` field in [`Cargo.toml`](../Cargo.toml)). CI builds the
[external consumer](development.md#external-consumer) on it.

## Using alongside libraries on a different OpenTelemetry version

Other libraries in your application (for example `openai-ergonomic`) may depend
on a different OpenTelemetry minor than this crate. They can coexist, as long as
the application meets this crate's MSRV, but Cargo will then resolve two
OpenTelemetry SDKs. Their exporter traits, providers, tracers, contexts, and
spans are distinct Rust types: do not pass this crate's exporter to a provider
from the other version, or pass its provider to an API that expects the other
version. Each OpenTelemetry version also has its own global provider; setting
one version's global provider does not configure the other version's
instrumentation. Use `cargo tree -i opentelemetry_sdk` to see which versions
are resolved; in an application that pulls in several, `cargo tree -d` and
`cargo tree -i opentelemetry_sdk@<version>` show which dependency uses each SDK.

Use separate tracing pipelines with each library's matching SDK, or explicitly
align dependencies in a separately reviewed integration. This crate does not
bridge the two versions or require an openai-ergonomic upgrade. The
[`LangfuseContext`](context.md) example integration illustrates attribute
conventions; it does not imply cross-version provider compatibility.
