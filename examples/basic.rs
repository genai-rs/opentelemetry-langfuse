//! Basic example of using opentelemetry-langfuse exporter.

use langfuse_ergonomic::client::LangfuseClient;
use opentelemetry::global;
use opentelemetry::trace::{Span, TraceContextExt, Tracer};
use opentelemetry::KeyValue;
use opentelemetry_langfuse::exporter_from_env;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load environment variables from .env file if present
    dotenvy::dotenv().ok();

    // Create the Langfuse exporter from environment variables
    // This requires:
    // - LANGFUSE_HOST: The base URL of your Langfuse instance
    // - LANGFUSE_PUBLIC_KEY: Your Langfuse public key
    // - LANGFUSE_SECRET_KEY: Your Langfuse secret key
    let exporter = exporter_from_env()?;

    // Create tracer provider with the Langfuse exporter
    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(Resource::builder().with_attributes(vec![
            KeyValue::new("service.name", "opentelemetry-langfuse-example"),
            KeyValue::new("service.version", "0.1.0"),
        ]).build())
        .build();

    // Set as global provider
    global::set_tracer_provider(provider);

    println!("Tracer initialized successfully!");

    // Get a tracer
    let tracer = global::tracer("example-tracer");

    // Create a span
    let mut span = tracer
        .span_builder("example-operation")
        .with_attributes(vec![
            KeyValue::new("operation.type", "demo"),
            KeyValue::new("operation.id", "12345"),
        ])
        .start(&tracer);

    // Add events to the span
    span.add_event(
        "Processing started",
        vec![KeyValue::new("item.count", 42i64)],
    );

    // Simulate some work
    sleep(Duration::from_millis(100)).await;

    // Add another event
    span.add_event(
        "Processing completed",
        vec![KeyValue::new("status", "success")],
    );

    // End the span
    span.end();

    println!("Span created and sent!");

    // Create nested spans
    {
        let parent_span = tracer.span_builder("parent-operation").start(&tracer);
        let cx = opentelemetry::Context::current_with_span(parent_span);
        let _guard = cx.attach();

        // Child span will automatically be linked to parent
        let mut child_span = tracer
            .span_builder("child-operation")
            .with_attributes(vec![KeyValue::new("child.id", "child-1")])
            .start(&tracer);

        sleep(Duration::from_millis(50)).await;
        child_span.end();

        // Another child
        let mut child_span2 = tracer
            .span_builder("child-operation")
            .with_attributes(vec![KeyValue::new("child.id", "child-2")])
            .start(&tracer);

        sleep(Duration::from_millis(50)).await;
        child_span2.end();
    }

    println!("Nested spans created!");

    // Give time for traces to be exported
    println!("Flushing traces...");
    sleep(Duration::from_secs(2)).await;

    // Provider will be shutdown when it goes out of scope
    // For clean shutdown, we'll just wait a bit more
    sleep(Duration::from_secs(1)).await;

    // Verify traces were sent to Langfuse
    println!("Verifying traces in Langfuse...");
    verify_traces_in_langfuse().await?;

    Ok(())
}

async fn verify_traces_in_langfuse() -> Result<(), Box<dyn Error>> {
    // Create Langfuse client using the same credentials
    let client = LangfuseClient::from_env()?;
    
    // Query for recent traces
    let traces = client.list_traces()
        .limit(10)
        .call()
        .await?;
    
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
