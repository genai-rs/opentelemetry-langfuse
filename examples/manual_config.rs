//! Example showing manual configuration without environment variables.

use opentelemetry::global;
use opentelemetry::trace::{Span, Tracer};
use opentelemetry_langfuse::TracerBuilder;
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load environment variables from .env file if present (for credentials)
    dotenv::dotenv().ok();

    // Manual configuration using the builder
    let tracer_provider = TracerBuilder::new("manual-config-example")
        .with_host("https://cloud.langfuse.com")
        .with_credentials(
            &std::env::var("LANGFUSE_PUBLIC_KEY").expect("LANGFUSE_PUBLIC_KEY not set"),
            &std::env::var("LANGFUSE_SECRET_KEY").expect("LANGFUSE_SECRET_KEY not set"),
        )
        .with_resource_attribute("environment", "development")
        .with_resource_attribute("version", "1.0.0")
        .install()?;

    println!("Tracer initialized with manual configuration!");

    // Get a tracer
    let tracer = global::tracer("manual-tracer");

    // Create a span with custom attributes
    let mut span = tracer
        .span_builder("manual-operation")
        .with_attributes(vec![
            opentelemetry::KeyValue::new("config.type", "manual"),
            opentelemetry::KeyValue::new("user.id", "user-123"),
            opentelemetry::KeyValue::new("session.id", "session-456"),
        ])
        .start(&tracer);

    // Add an event indicating an error was handled
    span.add_event(
        "error_handled",
        vec![
            opentelemetry::KeyValue::new(
                "error.message",
                "Something went wrong (but we handled it)",
            ),
            opentelemetry::KeyValue::new("error.handled", true),
        ],
    );

    // Set status
    span.set_status(opentelemetry::trace::Status::Ok);

    // Add structured event
    span.add_event(
        "user_action",
        vec![
            opentelemetry::KeyValue::new("action", "button_click"),
            opentelemetry::KeyValue::new("button.id", "submit"),
            opentelemetry::KeyValue::new("timestamp", chrono::Utc::now().to_rfc3339()),
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
    global::shutdown_tracer_provider();
    sleep(Duration::from_secs(1)).await;

    println!("Done! Check your Langfuse dashboard for the traces.");

    Ok(())
}

async fn do_something_that_might_fail<T: Tracer>(tracer: &T) -> Result<String, String> {
    let mut span = tracer
        .span_builder("risky-operation")
        .with_attributes(vec![opentelemetry::KeyValue::new("risk.level", "high")])
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
                opentelemetry::KeyValue::new("exception.type", "DatabaseError"),
                opentelemetry::KeyValue::new("exception.message", "Database connection timeout"),
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
