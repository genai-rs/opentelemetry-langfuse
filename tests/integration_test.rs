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

use chrono::Utc;
use langfuse_ergonomic::client::LangfuseClient;
use opentelemetry::trace::{Span, SpanKind, Tracer, TracerProvider};
use opentelemetry::KeyValue;
use opentelemetry_langfuse::exporter_from_env;
use opentelemetry_sdk::trace::{SdkTracerProvider, SimpleSpanProcessor};
use opentelemetry_sdk::Resource;
use serial_test::serial;
use std::time::Duration;
use tokio::time::sleep;

/// Helper to generate a unique test ID using timestamp
fn generate_test_id(test_name: &str) -> String {
    format!("test-{}-{}", test_name, Utc::now().timestamp_millis())
}

/// Helper to verify traces in Langfuse by searching for a specific test ID
async fn verify_trace_in_langfuse(test_id: &str) -> Result<bool, Box<dyn std::error::Error>> {
    println!("  Waiting for Langfuse to process traces...");

    // Wait a bit for Langfuse to process the traces
    sleep(Duration::from_secs(5)).await;

    println!("  Querying Langfuse API for test_id: {}", test_id);
    let client = LangfuseClient::from_env()?;

    // Query for recent traces with timeout
    let traces = tokio::time::timeout(
        Duration::from_secs(10),
        client.list_traces().limit(50).call()
    )
    .await
    .map_err(|_| "Timeout querying Langfuse API")??;

    println!("  Received response from Langfuse");

    // Check if we can find our trace by the test_id attribute
    if let Some(data) = traces.get("data") {
        if let Some(array) = data.as_array() {
            println!("  Found {} total traces", array.len());
            for trace in array {
                // Check if trace name or metadata contains our test_id
                if let Some(name) = trace.get("name").and_then(|v| v.as_str()) {
                    if name.contains(test_id) {
                        println!("  ✓ Found matching trace: {}", name);
                        return Ok(true);
                    }
                }
                // Also check metadata
                if let Some(metadata) = trace.get("metadata") {
                    let metadata_str = serde_json::to_string(metadata)?;
                    if metadata_str.contains(test_id) {
                        println!("  ✓ Found matching trace in metadata");
                        return Ok(true);
                    }
                }
            }
            println!("  ✗ No matching trace found for test_id: {}", test_id);
        }
    }

    Ok(false)
}

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_simple_span_processor() -> Result<(), Box<dyn std::error::Error>> {
    let test_id = generate_test_id("simple");

    // Create exporter with SimpleSpanProcessor (exports immediately, blocking)
    let exporter = exporter_from_env()?;
    let provider = SdkTracerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_attributes([
                    KeyValue::new("service.name", "integration-test-simple"),
                    KeyValue::new("test.id", test_id.clone()),
                ])
                .build(),
        )
        .with_span_processor(SimpleSpanProcessor::new(exporter))
        .build();

    // Use provider directly instead of global (to avoid conflicts between tests)
    let tracer = provider.tracer("integration-test");
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
        span.end();
    }

    // Shutdown provider to flush spans
    drop(provider);

    // Verify trace in Langfuse
    let found = verify_trace_in_langfuse(&test_id).await?;
    assert!(
        found,
        "Trace with test_id '{}' not found in Langfuse",
        test_id
    );

    Ok(())
}

// TODO: Enable when BatchSpanProcessor runtime issue is resolved
// See: https://github.com/open-telemetry/opentelemetry-rust/issues/XXXX
//
// The BatchSpanProcessor uses futures_executor::block_on in a background thread
// which cannot access the Tokio reactor in test context. This works fine in
// standalone examples (see examples/async_batch.rs) but fails in cargo test.
//
// #[tokio::test(flavor = "multi_thread")]
// #[serial]
// async fn test_batch_span_processor() -> Result<(), Box<dyn std::error::Error>> {
//     let test_id = generate_test_id("batch");
//
//     // Create exporter with BatchSpanProcessor (batches and exports periodically)
//     let exporter = exporter_from_env()?;
//     let provider = SdkTracerProvider::builder()
//         .with_resource(
//             Resource::builder()
//                 .with_attributes([
//                     KeyValue::new("service.name", "integration-test-batch"),
//                     KeyValue::new("test.id", test_id.clone()),
//                 ])
//                 .build(),
//         )
//         .with_batch_exporter(exporter)
//         .build();
//
//     // Use provider directly instead of global (to avoid conflicts between tests)
//     let tracer = provider.tracer("integration-test");
//
//     // Create multiple spans to test batching
//     for i in 0..5 {
//         let mut span = tracer
//             .span_builder(format!("{}-span-{}", test_id, i))
//             .with_kind(SpanKind::Server)
//             .with_attributes([
//                 KeyValue::new("test.type", "batch_processor"),
//                 KeyValue::new("test.timestamp", Utc::now().to_rfc3339()),
//                 KeyValue::new("batch.index", i as i64),
//             ])
//             .start(&tracer);
//
//         sleep(Duration::from_millis(10)).await;
//         span.set_attribute(KeyValue::new("test.status", "completed"));
//         span.end();
//     }
//
//     // Wait for batch export
//     sleep(Duration::from_secs(2)).await;
//
//     // Shutdown provider to flush remaining spans
//     drop(provider);
//
//     // Verify at least one trace in Langfuse
//     let found = verify_trace_in_langfuse(&test_id).await?;
//     assert!(
//         found,
//         "Trace with test_id '{}' not found in Langfuse",
//         test_id
//     );
//
//     Ok(())
// }
