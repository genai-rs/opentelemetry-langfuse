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
use opentelemetry::trace::{Span, SpanKind, Tracer};
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

#[tokio::test]
#[serial]
async fn test_simple_sync_export() -> Result<(), Box<dyn std::error::Error>> {
    let test_id = generate_test_id("simple-sync");

    // Create exporter with SimpleSpanProcessor
    let exporter = exporter_from_env()?;
    let provider = SdkTracerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_attributes([
                    KeyValue::new("service.name", "integration-test-simple-sync"),
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
                KeyValue::new("test.type", "simple_sync"),
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

#[tokio::test]
#[serial]
async fn test_simple_async_export() -> Result<(), Box<dyn std::error::Error>> {
    let test_id = generate_test_id("simple-async");

    // Create exporter with SimpleSpanProcessor
    let exporter = exporter_from_env()?;
    let provider = SdkTracerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_attributes([
                    KeyValue::new("service.name", "integration-test-simple-async"),
                    KeyValue::new("test.id", test_id.clone()),
                ])
                .build(),
        )
        .with_span_processor(SimpleSpanProcessor::new(exporter))
        .build();

    // Use provider directly instead of global (to avoid conflicts between tests)
    let tracer = provider.tracer("integration-test");
    {
        let mut root_span = tracer
            .span_builder(test_id.clone())
            .with_kind(SpanKind::Server)
            .with_attributes([
                KeyValue::new("test.type", "simple_async"),
                KeyValue::new("test.timestamp", Utc::now().to_rfc3339()),
            ])
            .start(&tracer);

        // Create child span
        {
            let mut child_span = tracer
                .span_builder("child-operation")
                .with_kind(SpanKind::Internal)
                .start(&tracer);

            sleep(Duration::from_millis(25)).await;
            child_span.set_attribute(KeyValue::new("child.status", "success"));
            child_span.end();
        }

        sleep(Duration::from_millis(25)).await;
        root_span.set_attribute(KeyValue::new("test.status", "completed"));
        root_span.end();
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

#[tokio::test]
#[serial]
async fn test_batch_sync_export() -> Result<(), Box<dyn std::error::Error>> {
    let test_id = generate_test_id("batch-sync");

    // Create exporter with BatchSpanProcessor
    let exporter = exporter_from_env()?;
    let provider = SdkTracerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_attributes([
                    KeyValue::new("service.name", "integration-test-batch-sync"),
                    KeyValue::new("test.id", test_id.clone()),
                ])
                .build(),
        )
        .with_batch_exporter(exporter)
        .build();

    // Use provider directly instead of global (to avoid conflicts between tests)
    let tracer = provider.tracer("integration-test");
    for i in 0..5 {
        let mut span = tracer
            .span_builder(format!("{}-batch-{}", test_id, i))
            .with_kind(SpanKind::Server)
            .with_attributes([
                KeyValue::new("test.type", "batch_sync"),
                KeyValue::new("test.timestamp", Utc::now().to_rfc3339()),
                KeyValue::new("batch.index", i as i64),
            ])
            .start(&tracer);

        sleep(Duration::from_millis(10)).await;
        span.set_attribute(KeyValue::new("test.status", "completed"));
        span.end();
    }

    // Wait for batch export
    sleep(Duration::from_secs(2)).await;

    // Shutdown provider to flush remaining spans
    drop(provider);

    // Verify at least one trace in Langfuse
    let found = verify_trace_in_langfuse(&test_id).await?;
    assert!(
        found,
        "Trace with test_id '{}' not found in Langfuse",
        test_id
    );

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_batch_async_export() -> Result<(), Box<dyn std::error::Error>> {
    let test_id = generate_test_id("batch-async");

    // Create exporter with BatchSpanProcessor
    let exporter = exporter_from_env()?;
    let provider = SdkTracerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_attributes([
                    KeyValue::new("service.name", "integration-test-batch-async"),
                    KeyValue::new("test.id", test_id.clone()),
                ])
                .build(),
        )
        .with_batch_exporter(exporter)
        .build();

    // Use provider directly instead of global (to avoid conflicts between tests)
    let tracer = provider.tracer("integration-test");

    // Create concurrent traces
    let mut handles = vec![];
    for i in 0..5 {
        let test_id = test_id.clone();
        let tracer = provider.tracer("integration-test");

        let handle = tokio::spawn(async move {
            let mut span = tracer
                .span_builder(format!("{}-async-{}", test_id, i))
                .with_kind(SpanKind::Server)
                .with_attributes([
                    KeyValue::new("test.type", "batch_async"),
                    KeyValue::new("test.timestamp", Utc::now().to_rfc3339()),
                    KeyValue::new("async.index", i as i64),
                ])
                .start(&tracer);

            // Create nested span
            {
                let mut child_span = tracer
                    .span_builder(format!("async-child-{}", i))
                    .with_kind(SpanKind::Internal)
                    .start(&tracer);

                sleep(Duration::from_millis(15)).await;
                child_span.set_attribute(KeyValue::new("child.status", "success"));
                child_span.end();
            }

            sleep(Duration::from_millis(15)).await;
            span.set_attribute(KeyValue::new("test.status", "completed"));
            span.end();
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await?;
    }

    // Wait for batch export
    sleep(Duration::from_secs(2)).await;

    // Shutdown provider to flush remaining spans
    drop(provider);

    // Verify at least one trace in Langfuse
    let found = verify_trace_in_langfuse(&test_id).await?;
    assert!(
        found,
        "Trace with test_id '{}' not found in Langfuse",
        test_id
    );

    Ok(())
}
