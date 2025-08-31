//! Example showing manual configuration without environment variables.

use langfuse_ergonomic::client::LangfuseClient;
use opentelemetry::global;
use opentelemetry::trace::{Span, Tracer};
use opentelemetry::KeyValue;
use opentelemetry_langfuse::ExporterBuilder;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load environment variables from .env file if present (for credentials)
    dotenvy::dotenv().ok();

    // Manual configuration using the builder
    let exporter = ExporterBuilder::new()
        .with_host("https://cloud.langfuse.com")
        .with_basic_auth(
            &std::env::var("LANGFUSE_PUBLIC_KEY").expect("LANGFUSE_PUBLIC_KEY not set"),
            &std::env::var("LANGFUSE_SECRET_KEY").expect("LANGFUSE_SECRET_KEY not set"),
        )
        .with_timeout(Duration::from_secs(10))
        .build()?;

    // Create tracer provider with the configured exporter
    let tracer_provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(
            Resource::builder()
                .with_attributes(vec![
                    KeyValue::new("service.name", "manual-config-example"),
                    KeyValue::new("environment", "development"),
                    KeyValue::new("version", "1.0.0"),
                ])
                .build(),
        )
        .build();

    // Set as global provider
    global::set_tracer_provider(tracer_provider.clone());

    println!("Tracer initialized with manual configuration!");

    // Get a tracer
    let tracer = global::tracer("manual-tracer");

    // Create a span with custom attributes
    let mut span = tracer
        .span_builder("manual-operation")
        .with_attributes(vec![
            KeyValue::new("config.type", "manual"),
            KeyValue::new("user.id", "user-123"),
            KeyValue::new("session.id", "session-456"),
        ])
        .start(&tracer);

    // Add an event indicating an error was handled
    span.add_event(
        "error_handled",
        vec![
            KeyValue::new("error.message", "Something went wrong (but we handled it)"),
            KeyValue::new("error.handled", true),
        ],
    );

    // Set status
    span.set_status(opentelemetry::trace::Status::Ok);

    // Add structured event
    span.add_event(
        "user_action",
        vec![
            KeyValue::new("action", "button_click"),
            KeyValue::new("button.id", "submit"),
            KeyValue::new("timestamp", chrono::Utc::now().to_rfc3339()),
        ],
    );

    sleep(Duration::from_millis(200)).await;

    span.end();

    println!("Custom span created!");

    // Example of error handling with spans
    let result = do_something_that_might_fail(&tracer).await;
    match result {
        Ok(value) => println!("Operation succeeded: {}", value),
        Err(e) => println!("Operation failed (as expected in demo): {}", e),
    }

    // Flush and shutdown
    println!("Flushing traces...");
    sleep(Duration::from_secs(2)).await;

    // Explicit shutdown using the provider
    drop(tracer_provider);
    // Provider will be shutdown when it goes out of scope
    sleep(Duration::from_secs(1)).await;

    // Verify traces were sent to Langfuse
    println!("Verifying traces in Langfuse...");
    verify_traces_in_langfuse().await?;

    Ok(())
}

async fn do_something_that_might_fail<T: Tracer>(tracer: &T) -> Result<String, String> {
    let mut span = tracer
        .span_builder("risky-operation")
        .with_attributes(vec![KeyValue::new("risk.level", "high")])
        .start(tracer);

    // Simulate some processing
    sleep(Duration::from_millis(100)).await;

    // Simulate an error condition
    let error_occurred = true;
    if error_occurred {
        span.set_status(opentelemetry::trace::Status::error("Operation failed"));
        span.add_event(
            "exception",
            vec![
                KeyValue::new("exception.type", "DatabaseError"),
                KeyValue::new("exception.message", "Database connection timeout"),
            ],
        );
        span.end();
        Err("Database connection timeout".to_string())
    } else {
        span.set_status(opentelemetry::trace::Status::Ok);
        span.end();
        Ok("Success".to_string())
    }
}

async fn verify_traces_in_langfuse() -> Result<(), Box<dyn Error>> {
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
                // Show first few trace IDs with details
                for (i, trace) in array.iter().take(3).enumerate() {
                    if let Some(id) = trace.get("id").and_then(|v| v.as_str()) {
                        print!("   {}. Trace ID: {}", i + 1, id);
                        if let Some(name) = trace.get("name").and_then(|v| v.as_str()) {
                            print!(" ({})", name);
                        }
                        println!();
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
