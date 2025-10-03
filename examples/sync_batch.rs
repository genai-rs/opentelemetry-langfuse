//! Example demonstrating batch processing in a mostly synchronous application.
//!
//! This example shows how to use BatchSpanProcessor when your application
//! is mostly synchronous but you want efficient batched exports.
//!
//! BatchSpanProcessor requires an async runtime to be available.
//! We create a runtime just for the span processing.
//!
//! Run with:
//! ```bash
//! export LANGFUSE_PUBLIC_KEY="pk-lf-..."
//! export LANGFUSE_SECRET_KEY="sk-lf-..."
//! export LANGFUSE_HOST="https://cloud.langfuse.com"
//!
//! cargo run --example sync_batch
//! ```

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use opentelemetry::global;
    use opentelemetry::trace::{Span, SpanKind, Tracer};
    use opentelemetry::KeyValue;
    use opentelemetry_langfuse::ExporterBuilder;
    use opentelemetry_sdk::trace::SdkTracerProvider;
    use opentelemetry_sdk::Resource;
    use std::thread;
    use std::time::Duration;

    println!("Creating Langfuse exporter with BatchSpanProcessor");
    println!("This batches spans and exports them periodically in the background.\n");

    // Create a runtime for the batch processor
    // This is needed because BatchSpanProcessor spawns background tasks
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(1) // Minimal threads for span processing
        .build()?;

    // Enter the runtime context for setup
    let _guard = runtime.enter();

    // Create the Langfuse exporter from environment variables
    let exporter = ExporterBuilder::from_env()?.build()?;

    // Build a tracer provider with BatchSpanProcessor
    // This will batch spans and export them periodically
    let provider = SdkTracerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_attributes([
                    KeyValue::new("service.name", "sync-batch-example"),
                    KeyValue::new("service.version", "1.0.0"),
                    KeyValue::new("deployment.environment", "production"),
                ])
                .build(),
        )
        .with_batch_exporter(exporter)
        .build();

    // Set as global provider
    global::set_tracer_provider(provider.clone());

    // Drop the guard - we can now use the tracer from synchronous code
    drop(_guard);

    // Create traces from synchronous code
    let tracer = global::tracer("sync-batch-example");

    println!("Creating spans (they will be batched)...");

    // Simulate a web server handling requests
    for request_id in 1..=10 {
        let mut request_span = tracer
            .span_builder("handle-request")
            .with_kind(SpanKind::Server)
            .with_attributes([
                KeyValue::new("http.method", "GET"),
                KeyValue::new("http.path", "/api/users"),
                KeyValue::new("request.id", request_id),
            ])
            .start(&tracer);

        // Simulate request processing
        thread::sleep(Duration::from_millis(10));

        // Nested span for database query
        {
            let mut db_span = tracer
                .span_builder("database-query")
                .with_kind(SpanKind::Client)
                .with_attributes([
                    KeyValue::new("db.system", "postgresql"),
                    KeyValue::new("db.operation", "SELECT"),
                ])
                .start(&tracer);

            thread::sleep(Duration::from_millis(5));
            db_span.set_attribute(KeyValue::new("db.rows_returned", 42));
            db_span.end();
        }

        request_span.set_attribute(KeyValue::new("http.status_code", 200));
        request_span.end();

        println!(
            "  - Request {} processed (span queued for batch export)",
            request_id
        );
    }

    println!("\nSpans are being batched and will be exported periodically.");
    println!("Waiting a moment for batch export to occur...");

    // Give the batch processor time to export
    thread::sleep(Duration::from_secs(2));

    println!("\nShutting down (this will flush any remaining spans)...");

    // Shutdown the provider - this flushes remaining spans
    drop(provider);

    println!("\n✅ All spans exported in batches!");
    println!("BatchSpanProcessor is ideal for production use:");
    println!("  - Better performance (less blocking)");
    println!("  - More efficient (fewer network calls)");
    println!("  - Automatic retries and error handling");
    println!("\nCheck your Langfuse dashboard for the traces.");

    // Verify traces were sent to Langfuse
    println!("\nVerifying traces in Langfuse...");
    runtime.block_on(verify_traces_in_langfuse())?;

    // Shutdown the runtime
    runtime.shutdown_timeout(Duration::from_secs(5));

    Ok(())
}

async fn verify_traces_in_langfuse() -> Result<(), Box<dyn std::error::Error>> {
    use langfuse_ergonomic::client::ClientBuilder;

    // Create Langfuse client using the same credentials
    let client = ClientBuilder::from_env()?.build()?;

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
