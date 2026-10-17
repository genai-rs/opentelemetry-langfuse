//! External application using only the documented OpenTelemetry dependency stack.
//! Run from the repository root with:
//! cargo +1.93.1 run --manifest-path examples/external-consumer/Cargo.toml

use opentelemetry::trace::{Span, Tracer, TracerProvider};
use opentelemetry_langfuse::ExporterBuilder;
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessor;
use opentelemetry_sdk::trace::{BatchConfigBuilder, SdkTracerProvider};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The fixture starts its own loopback endpoint and uses dummy credentials.
    let mut server = mockito::Server::new();
    let request = server
        .mock("POST", "/api/public/otel/v1/traces")
        .match_header("content-type", "application/x-protobuf")
        .match_request(|request| {
            request.body().is_ok_and(|body| {
                body.windows(b"external-consumer-span".len())
                    .any(|bytes| bytes == b"external-consumer-span")
            })
        })
        .with_status(200)
        .with_header("content-type", "application/x-protobuf")
        .with_body([])
        .expect(1)
        .create();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()?;
    let guard = runtime.enter();
    let exporter = ExporterBuilder::new()
        .with_host(&server.url())
        .with_basic_auth("pk-fixture", "sk-fixture")
        .with_timeout(Duration::from_secs(2))
        .build()?;
    let provider = SdkTracerProvider::builder()
        .with_span_processor(
            BatchSpanProcessor::builder(exporter, Tokio)
                .with_batch_config(
                    BatchConfigBuilder::default()
                        .with_max_export_timeout(Duration::from_secs(5))
                        .build(),
                )
                .build(),
        )
        .build();
    let tracer = provider.tracer("external-consumer");
    tracer.start("external-consumer-span").end();
    drop(tracer);
    drop(guard);

    // Shutdown flushes the queued span. Call from outside the runtime so its
    // worker can continue processing exports; no sleep or global provider needed.
    let shutdown = provider.shutdown_with_timeout(Duration::from_secs(5));
    runtime.shutdown_timeout(Duration::from_secs(5));
    shutdown?;
    request.assert();
    println!("Exported one span to the local endpoint and shut down successfully.");
    Ok(())
}
