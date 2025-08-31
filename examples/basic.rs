//! Basic example of using opentelemetry-langfuse.

use opentelemetry::global;
use opentelemetry::trace::{Span, TraceContextExt, Tracer};
use opentelemetry_langfuse::init_tracer_from_env;
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load environment variables from .env file if present
    dotenv::dotenv().ok();

    // Initialize tracer from environment variables
    // This requires:
    // - LANGFUSE_HOST: The base URL of your Langfuse instance
    // - LANGFUSE_PUBLIC_KEY: Your Langfuse public key
    // - LANGFUSE_SECRET_KEY: Your Langfuse secret key
    let _tracer_provider = init_tracer_from_env("opentelemetry-langfuse-example")?;

    println!("Tracer initialized successfully!");

    // Get a tracer
    let tracer = global::tracer("example-tracer");

    // Create a span
    let mut span = tracer
        .span_builder("example-operation")
        .with_attributes(vec![
            opentelemetry::KeyValue::new("operation.type", "demo"),
            opentelemetry::KeyValue::new("operation.id", "12345"),
        ])
        .start(&tracer);

    // Add events to the span
    span.add_event(
        "Processing started",
        vec![opentelemetry::KeyValue::new("item.count", 42i64)],
    );

    // Simulate some work
    sleep(Duration::from_millis(100)).await;

    // Add another event
    span.add_event(
        "Processing completed",
        vec![opentelemetry::KeyValue::new("status", "success")],
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
            .with_attributes(vec![opentelemetry::KeyValue::new("child.id", "child-1")])
            .start(&tracer);

        sleep(Duration::from_millis(50)).await;
        child_span.end();

        // Another child
        let mut child_span2 = tracer
            .span_builder("child-operation")
            .with_attributes(vec![opentelemetry::KeyValue::new("child.id", "child-2")])
            .start(&tracer);

        sleep(Duration::from_millis(50)).await;
        child_span2.end();
    }

    println!("Nested spans created!");

    // Give time for traces to be exported
    println!("Flushing traces...");
    sleep(Duration::from_secs(2)).await;

    // Shutdown the tracer provider
    global::shutdown_tracer_provider();
    sleep(Duration::from_secs(1)).await;

    println!("Done! Check your Langfuse dashboard for the traces.");

    Ok(())
}
