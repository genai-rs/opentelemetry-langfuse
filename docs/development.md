# Development

See also [CONTRIBUTING](../CONTRIBUTING.md).

## Tests

Unit tests, the local HTTP transport/query regression tests and the doctests
(including the code in these docs pages) do not need credentials:

```bash
cargo test --lib --test otlp_transport_test --test langfuse_query_test
cargo test --doc --all-features
```

The integration tests in [`tests/integration_test.rs`](../tests/integration_test.rs)
are marked `#[ignore]` because they need live Langfuse credentials. They verify
that traces are successfully exported to Langfuse and can be queried via the
Langfuse API. The tests cover:

- **SimpleSpanProcessor**: Immediate (blocking) export
- **BatchSpanProcessor (async runtime)**: Batched export with Tokio integration

To run them:

```bash
export LANGFUSE_PUBLIC_KEY="pk-lf-..."
export LANGFUSE_SECRET_KEY="sk-lf-..."
export LANGFUSE_HOST="https://cloud.langfuse.com"

cargo test --all-features -- --ignored
```

The live tests require Langfuse Cloud or self-hosted v4. They query the exact
exported trace ID, span ID, and name through Observations API v2, with a bounded
time range and a five-minute polling budget. They reuse the
[`langfuse-ergonomic`](https://github.com/genai-rs/langfuse-ergonomic) client's
authenticated transport, respect `Retry-After` on rate limits, and fail immediately
on authentication errors. Both ingestion and verification use `LANGFUSE_HOST`.
CI runs them in the `live-integration` job.

## External consumer

The [external consumer fixture](../examples/external-consumer) has its own manifest
and builds this crate as a path dependency, without the library's dev-dependencies.
CI compiles and runs it on the MSRV. It starts a local fake OTLP endpoint,
exports one span, verifies receipt, and explicitly shuts down the provider and
runtime. No Langfuse service or credentials are needed:

```bash
cargo check --manifest-path examples/external-consumer/Cargo.toml
cargo run --manifest-path examples/external-consumer/Cargo.toml
cargo tree --manifest-path examples/external-consumer/Cargo.toml -i opentelemetry_sdk
```

Prefix with `cargo +<msrv>` (see `rust-version` in [`Cargo.toml`](../Cargo.toml))
to reproduce the MSRV check. The fixture should resolve exactly one
OpenTelemetry SDK.

## CI and local CI

[`ci.yml`](../.github/workflows/ci.yml) calls the shared genai-rs
[`rust-ci.yml`](https://github.com/genai-rs/.github/blob/main/docs/rust-ci.md)
workflow and adds a `no-default-features` test job and the live integration
tests. Before opening a pull request, run the CI checks on your machine with
the shared [local-ci script](https://github.com/genai-rs/.github/blob/main/docs/local-ci.md);
a passing run attests the commit, and the PR then skips the heavy CI jobs:

```bash
bash ../.github/scripts/local-ci.sh --push   # with genai-rs/.github cloned next to this repo
```

Besides the shared checks it runs [`.github/local-ci-extra.sh`](../.github/local-ci-extra.sh)
(`cargo test --no-default-features`). The live integration tests run in CI
whether or not the commit is attested.
