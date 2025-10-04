//! Example demonstrating async batch processing with Tokio.
//!
//! This example shows how to use Langfuse exporters with BatchSpanProcessor
//! in a fully async application using Tokio.
//!
//! BatchSpanProcessor is recommended for production use because it:
//! - Batches spans for efficient network usage
//! - Exports asynchronously (non-blocking)
//! - Handles retries and errors gracefully
//!
//! Run with:
//! ```bash
//! export LANGFUSE_PUBLIC_KEY="pk-lf-..."
//! export LANGFUSE_SECRET_KEY="sk-lf-..."
//! export LANGFUSE_HOST="https://cloud.langfuse.com"
//!
//! cargo run --example async_batch
//! ```

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use opentelemetry::global;
    use opentelemetry::trace::{Span, SpanKind, Tracer};
    use opentelemetry::KeyValue;
    use opentelemetry_langfuse::ExporterBuilder;
    use opentelemetry_sdk::trace::SdkTracerProvider;
    use opentelemetry_sdk::Resource;
    use std::time::Duration;
    use tokio::time::sleep;

    println!("Creating Langfuse exporter with BatchSpanProcessor");
    println!("Running in async context with Tokio runtime.\n");

    // Create the Langfuse exporter from environment variables
    // This is just a standard OTLP exporter with Langfuse auth configured
    let exporter = ExporterBuilder::from_env()?.build()?;

    // Build a tracer provider with BatchSpanProcessor
    // The batch processor uses the Tokio runtime from #[tokio::main]
    let provider = SdkTracerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_attributes([
                    KeyValue::new("service.name", "async-batch-example"),
                    KeyValue::new("service.version", "2.0.0"),
                    KeyValue::new("deployment.environment", "production"),
                    KeyValue::new("runtime", "tokio"),
                ])
                .build(),
        )
        .with_batch_exporter(exporter)
        .build();

    // Set as global provider
    global::set_tracer_provider(provider.clone());

    // Create traces
    let tracer = global::tracer("async-service");

    println!("Simulating async web service handling requests...");

    // Simulate concurrent request handling
    let mut handles = vec![];
    let trace_id = std::sync::Arc::new(std::sync::Mutex::new(None));

    for request_id in 1..=5 {
        let tracer = global::tracer("async-service");
        let trace_id = trace_id.clone();

        let handle = tokio::spawn(async move {
            // Each request is handled in its own task
            let mut request_span = tracer
                .span_builder("handle-request")
                .with_kind(SpanKind::Server)
                .with_attributes([
                    KeyValue::new("http.method", "POST"),
                    KeyValue::new("http.path", "/api/process"),
                    KeyValue::new("request.id", request_id),
                ])
                .start(&tracer);

            // Capture the trace ID from the first request
            if request_id == 1 {
                let span_context = request_span.span_context();
                let tid = span_context.trace_id().to_string();
                *trace_id.lock().unwrap() = Some(tid);
            }

            // Simulate async I/O operation
            {
                let mut io_span = tracer
                    .span_builder("fetch-data")
                    .with_kind(SpanKind::Client)
                    .with_attributes([KeyValue::new("service.name", "external-api")])
                    .start(&tracer);

                // Simulate network call
                sleep(Duration::from_millis(50)).await;
                io_span.set_attribute(KeyValue::new("response.size", 1024));
                io_span.end();
            }

            // Simulate async processing
            {
                let mut process_span = tracer
                    .span_builder("process-data")
                    .with_kind(SpanKind::Internal)
                    .start(&tracer);

                sleep(Duration::from_millis(30)).await;
                process_span.set_attribute(KeyValue::new("items.processed", 100));
                process_span.end();
            }

            request_span.set_attribute(KeyValue::new("http.status_code", 200));
            request_span.end();

            println!(
                "  - Request {} completed (span queued for batch export)",
                request_id
            );
        });

        handles.push(handle);
    }

    // Wait for all requests to complete
    for handle in handles {
        handle.await?;
    }

    println!("\nSimulating background tasks...");

    // Simulate background processing
    {
        let mut bg_span = tracer
            .span_builder("background-job")
            .with_kind(SpanKind::Internal)
            .with_attributes([
                KeyValue::new("job.type", "cleanup"),
                KeyValue::new("job.scheduled", true),
            ])
            .start(&tracer);

        // Simulate multiple async operations within the job
        for i in 0..3 {
            let mut task_span = tracer
                .span_builder("cleanup-task")
                .with_kind(SpanKind::Internal)
                .with_attributes([KeyValue::new("task.id", i)])
                .start(&tracer);

            sleep(Duration::from_millis(20)).await;
            task_span.set_attribute(KeyValue::new("task.status", "completed"));
            task_span.end();
        }

        bg_span.set_attribute(KeyValue::new("job.status", "success"));
        bg_span.end();
        println!("  - Background job completed");
    }

    println!("\nWaiting for batch export...");
    sleep(Duration::from_secs(1)).await;

    // Shutdown to ensure all spans are flushed
    println!("Shutting down (flushing remaining spans)...");
    drop(provider);

    println!("\nAll spans exported in batches!");
    println!("Benefits of async + batch processing:");
    println!("  - Non-blocking span export");
    println!("  - Efficient network usage");
    println!("  - Better performance for high-throughput applications");
    println!("\nCheck your Langfuse dashboard for the traces.");

    // Verify traces were sent to Langfuse
    println!("\nVerifying traces in Langfuse...");
    let expected_trace_id = trace_id.lock().unwrap().clone().unwrap_or_default();
    verify_traces_in_langfuse(&expected_trace_id).await?;

    Ok(())
}

async fn verify_traces_in_langfuse(
    expected_trace_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use langfuse_ergonomic::client::ClientBuilder;

    // Create Langfuse client using the same credentials
    let client = ClientBuilder::from_env()?.build()?;

    // Query for recent traces
    let traces = client.list_traces().limit(10).call().await?;

    // The response is now a strongly-typed Traces struct
    if traces.data.is_empty() {
        println!("WARNING: No traces found in Langfuse yet. They may still be processing.");
        return Ok(());
    }

    println!("Found {} traces in Langfuse!", traces.data.len());

    // Verify the expected trace ID is present
    let found_expected = traces.data.iter().any(|trace| trace.id == expected_trace_id);

    if found_expected {
        println!("SUCCESS: Found expected trace ID: {}", expected_trace_id);
    } else {
        println!("WARNING: Expected trace ID {} not found yet. Recent trace IDs:", expected_trace_id);
        for (i, trace) in traces.data.iter().take(3).enumerate() {
            println!("   {}. Trace ID: {}", i + 1, trace.id);
        }
    }

    Ok(())
}
