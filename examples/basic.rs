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
    dotenvy::dotenv().ok();

    let exporter = exporter_from_env()?;

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(
            Resource::builder()
                .with_attributes(vec![
                    KeyValue::new("service.name", "opentelemetry-langfuse-example"),
                    KeyValue::new("service.version", "0.1.0"),
                ])
                .build(),
        )
        .build();

    global::set_tracer_provider(provider);

    println!("Tracer initialized successfully!");

    let tracer = global::tracer("example-tracer");

    let mut span = tracer
        .span_builder("example-operation")
        .with_attributes(vec![
            KeyValue::new("operation.type", "demo"),
            KeyValue::new("operation.id", "12345"),
        ])
        .start(&tracer);

    span.add_event(
        "Processing started",
        vec![KeyValue::new("item.count", 42i64)],
    );

    sleep(Duration::from_millis(100)).await;

    span.add_event(
        "Processing completed",
        vec![KeyValue::new("status", "success")],
    );

    span.end();

    println!("Span created and sent!");

    {
        let parent_span = tracer.span_builder("parent-operation").start(&tracer);
        let cx = opentelemetry::Context::current_with_span(parent_span);
        let _guard = cx.attach();

        let mut child_span = tracer
            .span_builder("child-operation")
            .with_attributes(vec![KeyValue::new("child.id", "child-1")])
            .start(&tracer);

        sleep(Duration::from_millis(50)).await;
        child_span.end();

        let mut child_span2 = tracer
            .span_builder("child-operation")
            .with_attributes(vec![KeyValue::new("child.id", "child-2")])
            .start(&tracer);

        sleep(Duration::from_millis(50)).await;
        child_span2.end();
    }

    println!("Nested spans created!");

    println!("Flushing traces...");
    sleep(Duration::from_secs(2)).await;

    sleep(Duration::from_secs(1)).await;

    println!("Verifying traces in Langfuse...");
    verify_traces_in_langfuse().await?;

    Ok(())
}

async fn verify_traces_in_langfuse() -> Result<(), Box<dyn Error>> {
    let client = LangfuseClient::from_env()?;

    let traces = client.list_traces().limit(10).call().await?;

    if let Some(data) = traces.get("data") {
        if let Some(array) = data.as_array() {
            if array.is_empty() {
                println!("⚠️  No traces found in Langfuse yet. They may still be processing.");
            } else {
                println!("✅ Found {} traces in Langfuse!", array.len());
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
