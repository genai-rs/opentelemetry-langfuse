//! Exercise the OTLP/HTTP transport without requiring a Langfuse account.

use opentelemetry::trace::{Span, Tracer, TracerProvider};
use opentelemetry_langfuse::{build_auth_header, ExporterBuilder};
use opentelemetry_sdk::trace::{SdkTracerProvider, SpanExporter};
use std::time::{Duration, SystemTime};

#[tokio::test]
async fn export_span_with_default_and_custom_reqwest_clients(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut server = mockito::Server::new_async().await;
    let auth = build_auth_header("pk-transport-test", "sk-transport-test");
    let request = server
        .mock("POST", "/api/public/otel/v1/traces")
        .match_header("authorization", auth.as_str())
        .match_header("content-type", "application/x-protobuf")
        .match_header("x-service-version", "dependency-upgrade")
        .match_request(|request| {
            request.body().is_ok_and(|body| {
                body.windows(b"transport-upgrade-span".len())
                    .any(|bytes| bytes == b"transport-upgrade-span")
            })
        })
        .with_status(200)
        .with_header("content-type", "application/x-protobuf")
        .with_body([])
        .expect(2)
        .create_async()
        .await;

    let provider = SdkTracerProvider::builder().build();
    let tracer = provider.tracer("transport-upgrade-test");
    for custom_client in [false, true] {
        let mut builder = ExporterBuilder::new()
            .with_host(&server.url())
            // Explicit credentials must override an additional auth header.
            .with_header("authorization", "Bearer obsolete")
            .with_basic_auth("pk-transport-test", "sk-transport-test")
            .with_header("x-service-version", "dependency-upgrade")
            .with_timeout(Duration::from_secs(5));
        if custom_client {
            builder = builder.with_http_client(reqwest::Client::builder().no_proxy().build()?);
        }
        let exporter = builder.build()?;
        let mut span = tracer.start("transport-upgrade-span");
        let mut data = span.exported_data().expect("span must be sampled");
        data.end_time = SystemTime::now();
        span.end();
        tokio::time::timeout(Duration::from_secs(10), exporter.export(vec![data])).await??;
    }

    request.assert_async().await;
    provider.shutdown()?;
    Ok(())
}
