//! Integration tests for OpenTelemetry Langfuse exporter.
//!
//! These tests verify that traces are successfully exported to Langfuse
//! and can be queried via the Langfuse API.
//!
//! Tests run serially to avoid interference between concurrent test runs.
//!
//! Run with:
//! ```bash
//! export LANGFUSE_PUBLIC_KEY="pk-lf-..."
//! export LANGFUSE_SECRET_KEY="sk-lf-..."
//! export LANGFUSE_HOST="https://cloud.langfuse.com"
//!
//! cargo test --test integration_test
//! ```

mod support;

use chrono::Utc;
use langfuse_ergonomic::client::ClientBuilder;
use opentelemetry::trace::{Span, SpanKind, Tracer, TracerProvider};
use opentelemetry::KeyValue;
use opentelemetry_langfuse::ExporterBuilder;
use opentelemetry_sdk::trace::{
    span_processor_with_async_runtime::BatchSpanProcessor, SdkTracerProvider, SimpleSpanProcessor,
};
use opentelemetry_sdk::{runtime::Tokio, Resource};
use serial_test::serial;
use std::time::Duration;
use tokio::time::sleep;

/// Helper to generate a unique test ID using timestamp and platform
fn generate_test_id(test_name: &str) -> String {
    let platform = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    format!(
        "test-{}-{}-{}-{}",
        test_name,
        platform,
        arch,
        Utc::now().timestamp_millis()
    )
}

/// Verify the exact exported span, without scanning unrelated project traces.
async fn verify_trace_in_langfuse(
    query: &support::ObservationQuery,
) -> Result<(), Box<dyn std::error::Error>> {
    let host =
        std::env::var("LANGFUSE_HOST").unwrap_or_else(|_| "https://cloud.langfuse.com".to_string());
    let client = ClientBuilder::from_env()?.base_url(host).build()?;
    support::verify_observation(
        &client,
        query,
        Duration::from_secs(300),
        Duration::from_secs(30),
    )
    .await
}

fn observation_query(span: &impl Span, name: &str) -> support::ObservationQuery {
    let now = Utc::now();
    support::ObservationQuery {
        trace_id: span.span_context().trace_id().to_string(),
        span_id: span.span_context().span_id().to_string(),
        name: name.to_string(),
        from_start_time: (now - chrono::Duration::minutes(1)).to_rfc3339(),
        to_start_time: (now + chrono::Duration::minutes(1)).to_rfc3339(),
    }
}

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_simple_span_processor() -> Result<(), Box<dyn std::error::Error>> {
    let test_id = generate_test_id("simple");

    // Create exporter with SimpleSpanProcessor (exports immediately, blocking)
    let exporter = ExporterBuilder::from_env()?
        // Required for real-time visibility in the Cloud/v4 Observations API v2.
        .with_header("x-langfuse-ingestion-version", "4")
        .build()?;
    let provider = SdkTracerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_attributes([
                    KeyValue::new("service.name", "integration-test-simple"),
                    KeyValue::new("test.id", test_id.clone()),
                    KeyValue::new("test.platform", std::env::consts::OS),
                    KeyValue::new("test.arch", std::env::consts::ARCH),
                ])
                .build(),
        )
        .with_span_processor(SimpleSpanProcessor::new(exporter))
        .build();

    // Use provider directly instead of global (to avoid conflicts between tests)
    let tracer = provider.tracer("integration-test");
    let query;
    {
        let mut span = tracer
            .span_builder(test_id.clone())
            .with_kind(SpanKind::Server)
            .with_attributes([
                KeyValue::new("test.type", "simple_processor"),
                KeyValue::new("test.timestamp", Utc::now().to_rfc3339()),
            ])
            .start(&tracer);

        sleep(Duration::from_millis(50)).await;
        span.set_attribute(KeyValue::new("test.status", "completed"));
        query = observation_query(&span, &test_id);
        span.end();
    }

    // Shutdown provider to flush spans
    provider.shutdown()?;

    // Verify trace in Langfuse
    verify_trace_in_langfuse(&query).await?;

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_batch_span_processor() -> Result<(), Box<dyn std::error::Error>> {
    let test_id = generate_test_id("batch");

    // Create exporter with BatchSpanProcessor (async runtime version)
    // This uses the experimental span_processor_with_async_runtime module
    // which properly integrates with Tokio runtime
    let exporter = ExporterBuilder::from_env()?
        // Required for real-time visibility in the Cloud/v4 Observations API v2.
        .with_header("x-langfuse-ingestion-version", "4")
        .build()?;
    let provider = SdkTracerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_attributes([
                    KeyValue::new("service.name", "integration-test-batch"),
                    KeyValue::new("test.id", test_id.clone()),
                    KeyValue::new("test.platform", std::env::consts::OS),
                    KeyValue::new("test.arch", std::env::consts::ARCH),
                ])
                .build(),
        )
        .with_span_processor(BatchSpanProcessor::builder(exporter, Tokio).build())
        .build();

    // Use provider directly instead of global (to avoid conflicts between tests)
    let tracer = provider.tracer("integration-test");
    let query;
    {
        let mut span = tracer
            .span_builder(test_id.clone())
            .with_kind(SpanKind::Server)
            .with_attributes([
                KeyValue::new("test.type", "batch_processor"),
                KeyValue::new("test.timestamp", Utc::now().to_rfc3339()),
            ])
            .start(&tracer);

        sleep(Duration::from_millis(50)).await;
        span.set_attribute(KeyValue::new("test.status", "completed"));
        query = observation_query(&span, &test_id);
        span.end();
    }

    // Shutdown provider to flush spans
    provider.shutdown()?;

    // Verify trace in Langfuse
    verify_trace_in_langfuse(&query).await?;

    Ok(())
}
