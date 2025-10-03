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

/// Helper to verify traces in Langfuse by searching for a specific test ID
/// Polls Langfuse API with retries to handle eventual consistency
async fn verify_trace_in_langfuse(test_id: &str) -> Result<bool, Box<dyn std::error::Error>> {
    println!("  Polling Langfuse API for trace with test_id: {}", test_id);

    let client = LangfuseClient::from_env()?;

    // Retry configuration: poll up to 40 times with 3 second delays
    // This gives Langfuse up to 2 minutes to process the trace
    // Handles slow processing on different platforms and eventual consistency
    const MAX_ATTEMPTS: u32 = 40;
    const RETRY_DELAY_SECS: u64 = 3;

    for attempt in 1..=MAX_ATTEMPTS {
        println!(
            "  Attempt {}/{}: Querying Langfuse API...",
            attempt, MAX_ATTEMPTS
        );

        // Query for recent traces with timeout
        let traces = match tokio::time::timeout(
            Duration::from_secs(10),
            client.list_traces().limit(50).call(),
        )
        .await
        {
            Ok(Ok(traces)) => traces,
            Ok(Err(e)) => {
                println!("  ⚠ API error on attempt {}: {}", attempt, e);
                if attempt < MAX_ATTEMPTS {
                    sleep(Duration::from_secs(RETRY_DELAY_SECS)).await;
                    continue;
                }
                return Err(e.into());
            }
            Err(_) => {
                println!("  ⚠ Timeout on attempt {}", attempt);
                if attempt < MAX_ATTEMPTS {
                    sleep(Duration::from_secs(RETRY_DELAY_SECS)).await;
                    continue;
                }
                return Err("Timeout querying Langfuse API".into());
            }
        };

        // Check if we can find our trace by the test_id attribute
        // The response is now a strongly-typed Traces struct
        println!("  Found {} total traces in response", traces.data.len());

        for trace in &traces.data {
            // Check if trace name contains our test_id
            if let Some(Some(name)) = &trace.name {
                if name.contains(test_id) {
                    println!("  ✓ Found matching trace: {} (attempt {})", name, attempt);
                    return Ok(true);
                }
            }
            // Also check metadata
            if let Some(Some(metadata)) = &trace.metadata {
                let metadata_str = serde_json::to_string(metadata)?;
                if metadata_str.contains(test_id) {
                    println!("  ✓ Found matching trace in metadata (attempt {})", attempt);
                    return Ok(true);
                }
            }
        }

        if attempt < MAX_ATTEMPTS {
            println!(
                "  ✗ Trace not found yet, waiting {} seconds before retry...",
                RETRY_DELAY_SECS
            );
            sleep(Duration::from_secs(RETRY_DELAY_SECS)).await;
        }
    }

    println!("  ✗ Trace not found after {} attempts", MAX_ATTEMPTS);
    Ok(false)
}

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_simple_span_processor() -> Result<(), Box<dyn std::error::Error>> {
    let test_id = generate_test_id("simple");

    // Create exporter with SimpleSpanProcessor (exports immediately, blocking)
    let exporter = ExporterBuilder::from_env()?.build()?;
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

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_batch_span_processor() -> Result<(), Box<dyn std::error::Error>> {
    let test_id = generate_test_id("batch");

    // Create exporter with BatchSpanProcessor (async runtime version)
    // This uses the experimental span_processor_with_async_runtime module
    // which properly integrates with Tokio runtime
    let exporter = ExporterBuilder::from_env()?.build()?;
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
        span.end();
    }

    // Shutdown provider to flush spans
    let _ = provider.shutdown();

    // Verify trace in Langfuse
    let found = verify_trace_in_langfuse(&test_id).await?;
    assert!(
        found,
        "Trace with test_id '{}' not found in Langfuse",
        test_id
    );

    Ok(())
}
