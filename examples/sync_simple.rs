//! Example demonstrating synchronous tracing with SimpleSpanProcessor.
//!
//! This example shows how to use SimpleSpanProcessor which exports spans
//! immediately when they end (blocking). Note that HTTP exporters always
//! require an async runtime, even with SimpleSpanProcessor.
//!
//! This is suitable for:
//! - Development and testing
//! - Low-throughput applications
//! - When you need immediate export
//!
//! Run with:
//! ```bash
//! export LANGFUSE_PUBLIC_KEY="pk-lf-..."
//! export LANGFUSE_SECRET_KEY="sk-lf-..."
//! export LANGFUSE_HOST="https://cloud.langfuse.com"
//!
//! cargo run --example sync_simple
//! ```

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use opentelemetry::global;
    use opentelemetry::trace::{Span, SpanKind, Tracer};
    use opentelemetry::KeyValue;
    use opentelemetry_langfuse::exporter_from_env;
    use opentelemetry_sdk::trace::{SdkTracerProvider, SimpleSpanProcessor};
    use opentelemetry_sdk::Resource;
    use std::time::Duration;
    use tokio::time;

    println!("Creating Langfuse exporter with SimpleSpanProcessor");
    println!("This exports spans immediately when they end (blocking).\n");

    // Create the Langfuse exporter from environment variables
    let exporter = exporter_from_env()?;

    // Build a tracer provider with SimpleSpanProcessor
    // Note: HTTP export requires async runtime even with SimpleSpanProcessor
    let provider = SdkTracerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_attributes([
                    KeyValue::new("service.name", "sync-simple-example"),
                    KeyValue::new("service.version", "1.0.0"),
                ])
                .build(),
        )
        .with_span_processor(SimpleSpanProcessor::new(exporter))
        .build();

    // Set as global provider
    global::set_tracer_provider(provider.clone());

    // Create some traces
    let tracer = global::tracer("sync-example");

    println!("Creating root span...");
    {
        let mut root_span = tracer
            .span_builder("process-order")
            .with_kind(SpanKind::Server)
            .with_attributes([
                KeyValue::new("order.id", "12345"),
                KeyValue::new("customer.type", "premium"),
            ])
            .start(&tracer);

        // Simulate some work
        time::sleep(Duration::from_millis(50)).await;

        // Create nested spans
        println!("Creating child spans...");
        {
            let mut validate_span = tracer
                .span_builder("validate-order")
                .with_kind(SpanKind::Internal)
                .start(&tracer);

            time::sleep(Duration::from_millis(20)).await;
            validate_span.set_attribute(KeyValue::new("validation.passed", true));
            validate_span.end();
            println!("  - validate-order span exported immediately");
        }

        {
            let mut payment_span = tracer
                .span_builder("process-payment")
                .with_kind(SpanKind::Internal)
                .with_attributes([
                    KeyValue::new("payment.method", "credit_card"),
                    KeyValue::new("payment.amount", 99.99),
                ])
                .start(&tracer);

            time::sleep(Duration::from_millis(30)).await;
            payment_span.set_attribute(KeyValue::new("payment.status", "success"));
            payment_span.end();
            println!("  - process-payment span exported immediately");
        }

        root_span.set_attribute(KeyValue::new("order.status", "completed"));
        root_span.end();
        println!("  - process-order span exported immediately");
    }

    println!("\nCreating another trace...");
    {
        let mut span = tracer
            .span_builder("background-task")
            .with_kind(SpanKind::Internal)
            .start(&tracer);

        time::sleep(Duration::from_millis(25)).await;
        span.set_attribute(KeyValue::new("task.result", "success"));
        span.end();
        println!("  - background-task span exported immediately");
    }

    // Shutdown the provider
    drop(provider);

    println!("\n✅ All spans exported synchronously!");
    println!("SimpleSpanProcessor is great for development and low-throughput scenarios.");
    println!("For production, consider using BatchSpanProcessor (see sync_batch or async_batch examples).");
    println!("\nCheck your Langfuse dashboard for the traces.");

    // Verify traces were sent to Langfuse
    println!("\nVerifying traces in Langfuse...");
    verify_traces_in_langfuse().await?;

    Ok(())
}

async fn verify_traces_in_langfuse() -> Result<(), Box<dyn std::error::Error>> {
    use langfuse_ergonomic::client::LangfuseClient;

    // Create Langfuse client using the same credentials
    let client = LangfuseClient::from_env()?;

    // Query for recent traces
    let traces = client.list_traces().limit(10).call().await?;

    // The response is a JSON value, so we check if it contains data
    if let Some(data) = traces.get("data") {
        if let Some(array) = data.as_array() {
            if array.is_empty() {
                println!("⚠️  No traces found in Langfuse yet. They may still be processing.");
            } else {
                println!("✅ Found {} traces in Langfuse!", array.len());
                // Show first few trace IDs
                for (i, trace) in array.iter().take(3).enumerate() {
                    if let Some(id) = trace.get("id").and_then(|v| v.as_str()) {
                        println!("   {}. Trace ID: {}", i + 1, id);
                    }
                }
            }
        } else {
            println!("✅ Successfully connected to Langfuse API");
        }
    } else {
        println!("⚠️  Unexpected response format from Langfuse");
    }

    Ok(())
}
