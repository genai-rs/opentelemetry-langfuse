//! Example demonstrating explicit context passing without global state.
//!
//! This example shows how to pass TracingContext through your application
//! layers without relying on global variables or thread-local storage.

use opentelemetry::trace::{Span, SpanKind, Tracer};
use opentelemetry::KeyValue;
use opentelemetry_langfuse::{LangfuseTracerBuilder, TracingContext};
use opentelemetry_sdk::runtime::Tokio;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

/// Application service that processes requests
struct LLMService {
    tracer: Box<dyn Tracer + Send + Sync>,
}

impl LLMService {
    fn new(tracer: impl Tracer + Send + Sync + 'static) -> Self {
        Self {
            tracer: Box::new(tracer),
        }
    }

    /// Process a request with explicit context
    async fn process_request(&self, request: &str, context: &TracingContext) -> String {
        let mut span = self
            .tracer
            .span_builder("process_request")
            .with_kind(SpanKind::Internal)
            .with_attributes(context.to_otel_attributes())
            .with_attributes(vec![KeyValue::new("request.content", request.to_string())])
            .start(&*self.tracer);

        sleep(Duration::from_millis(100)).await;

        let response = self.call_llm(request, context).await;

        span.set_attribute(KeyValue::new("response.content", response.clone()));
        span.end();

        response
    }

    /// Call LLM with context
    async fn call_llm(&self, prompt: &str, context: &TracingContext) -> String {
        let llm_context = context.child().with_model("gpt-4").with_temperature(0.7);

        let mut span = self
            .tracer
            .span_builder("llm_call")
            .with_kind(SpanKind::Client)
            .with_attributes(llm_context.to_otel_attributes())
            .with_attributes(vec![
                KeyValue::new("gen_ai.prompt.0.role", "user"),
                KeyValue::new("gen_ai.prompt.0.content", prompt.to_string()),
            ])
            .start(&*self.tracer);

        sleep(Duration::from_millis(200)).await;
        let response = format!("Response to: {}", prompt);

        span.set_attribute(KeyValue::new(
            "gen_ai.completion.0.content",
            response.clone(),
        ));
        span.set_attribute(KeyValue::new("gen_ai.usage.prompt_tokens", 10i64));
        span.set_attribute(KeyValue::new("gen_ai.usage.completion_tokens", 15i64));

        span.end();

        response
    }
}

/// Request handler that creates context for each request
async fn handle_user_request(
    service: &LLMService,
    user_id: &str,
    session_id: &str,
    request: &str,
) -> String {
    let context = TracingContext::new()
        .with_user(user_id)
        .with_session(session_id)
        .with_metadata("request_id", json!(uuid::Uuid::new_v4().to_string()))
        .with_metadata("timestamp", json!(chrono::Utc::now().to_rfc3339()));

    println!(
        "Processing request for user: {}, session: {}",
        user_id, session_id
    );

    service.process_request(request, &context).await
}

/// Batch processing with shared context
async fn process_batch(service: &LLMService, items: Vec<String>, batch_context: &TracingContext) {
    println!("\nProcessing batch of {} items", items.len());

    for (i, item) in items.iter().enumerate() {
        let item_context = batch_context
            .child()
            .with_metadata("batch_item_index", json!(i))
            .with_metadata("item_content", json!(item));

        let response = service.process_request(item, &item_context).await;
        println!("  Item {}: {} -> {}", i, item, response);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let tracer = LangfuseTracerBuilder::new(Tokio)
        .with_service_name("context-passing-demo")
        .with_endpoint(
            std::env::var("LANGFUSE_ENDPOINT")
                .unwrap_or_else(|_| "https://cloud.langfuse.com/api/public/otel".to_string()),
        )
        .build()?;

    let service = LLMService::new(tracer);

    println!("=== Context Passing Demo ===\n");

    println!("Example 1: Individual requests with unique contexts");

    let response1 = handle_user_request(
        &service,
        "user-001",
        "session-aaa",
        "What is the capital of France?",
    )
    .await;
    println!("Response 1: {}\n", response1);

    let response2 =
        handle_user_request(&service, "user-002", "session-bbb", "What is 2 + 2?").await;
    println!("Response 2: {}\n", response2);

    println!("Example 2: Batch processing with shared context");

    let batch_context = TracingContext::new()
        .with_name("batch-processing")
        .with_metadata("batch_id", json!("batch-123"))
        .with_metadata("batch_type", json!("questions"));

    let batch_items = vec![
        "What is Rust?".to_string(),
        "Explain OpenTelemetry".to_string(),
        "What is Langfuse?".to_string(),
    ];

    process_batch(&service, batch_items, &batch_context).await;

    println!("\nExample 3: Context inheritance chain");

    let root_context = TracingContext::new()
        .with_name("root-operation")
        .with_metadata("level", json!("root"));

    let child_context = root_context
        .child()
        .with_metadata("level", json!("child"))
        .with_metadata("child_specific", json!("data"));

    let grandchild_context = child_context
        .child()
        .with_metadata("level", json!("grandchild"));

    println!(
        "Root attributes: {:?}",
        root_context.get_all_attributes().len()
    );
    println!(
        "Child attributes: {:?}",
        child_context.get_all_attributes().len()
    );
    println!(
        "Grandchild attributes: {:?}",
        grandchild_context.get_all_attributes().len()
    );

    let response = service
        .process_request("Test with inherited context", &grandchild_context)
        .await;
    println!("Response with inherited context: {}", response);

    println!("\n=== Demo Complete ===");

    sleep(Duration::from_secs(2)).await;

    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}

