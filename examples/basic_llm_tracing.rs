//! Basic example of LLM tracing with OpenTelemetry and Langfuse.
//!
//! This example demonstrates how to instrument LLM calls with proper
//! OpenTelemetry GenAI semantic conventions and send them to Langfuse.

use opentelemetry::trace::{Span, SpanKind, Tracer};
use opentelemetry::KeyValue;
use opentelemetry_langfuse::{LangfuseTracerBuilder, TracingContext};
use opentelemetry_sdk::runtime::Tokio;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let endpoint = std::env::var("LANGFUSE_ENDPOINT")
        .unwrap_or_else(|_| "https://cloud.langfuse.com/api/public/otel".to_string());

    let api_key = std::env::var("LANGFUSE_API_KEY").unwrap_or_else(|_| {
        println!("Warning: LANGFUSE_API_KEY not set. Using demo key.");
        "demo-key".to_string()
    });

    let context = TracingContext::new()
        .with_session("demo-session-123")
        .with_user("demo-user-456")
        .with_metadata("environment", json!("development"))
        .with_metadata("version", json!("1.0.0"));

    let tracer = LangfuseTracerBuilder::new(Tokio)
        .with_endpoint(endpoint)
        .with_api_key(api_key)
        .with_service_name("llm-demo-service")
        .with_service_version("1.0.0")
        .with_context(context.clone())
        .build()?;

    println!("Starting LLM tracing demo...");

    for i in 1..=3 {
        println!("\n--- Interaction {} ---", i);

        let mut conversation_span = tracer
            .span_builder(format!("conversation_{}", i))
            .with_kind(SpanKind::Server)
            .with_attributes(vec![
                KeyValue::new("langfuse.trace.name", format!("Demo Conversation {}", i)),
                KeyValue::new("conversation.turn", i as i64),
            ])
            .start(&tracer);

        let user_prompt = format!("Tell me a fact about the number {}", i);
        conversation_span.set_attribute(KeyValue::new("user.input", user_prompt.clone()));

        let mut llm_span = tracer
            .span_builder("llm.completion")
            .with_kind(SpanKind::Client)
            .with_attributes(vec![
                KeyValue::new("gen_ai.system", "openai"),
                KeyValue::new("gen_ai.request.model", "gpt-4"),
                KeyValue::new("gen_ai.request.temperature", 0.7),
                KeyValue::new("gen_ai.request.max_tokens", 150),
                KeyValue::new("gen_ai.request.top_p", 0.9),
                KeyValue::new("gen_ai.prompt.0.role", "system"),
                KeyValue::new(
                    "gen_ai.prompt.0.content",
                    "You are a helpful assistant that provides interesting facts about numbers.",
                ),
                KeyValue::new("gen_ai.prompt.1.role", "user"),
                KeyValue::new("gen_ai.prompt.1.content", user_prompt.clone()),
                KeyValue::new("langfuse.observation.type", "generation"),
                KeyValue::new("langfuse.observation.model.name", "gpt-4"),
            ])
            .start(&tracer);

        println!("Processing LLM request...");
        sleep(Duration::from_millis(500)).await;

        let completion = format!(
            "The number {} is {}",
            i,
            match i {
                1 => "the first positive integer and the multiplicative identity.",
                2 => "the only even prime number.",
                3 => "the first odd prime number.",
                _ => "a natural number.",
            }
        );

        llm_span.set_attribute(KeyValue::new("gen_ai.completion.0.role", "assistant"));
        llm_span.set_attribute(KeyValue::new(
            "gen_ai.completion.0.content",
            completion.clone(),
        ));

        let prompt_tokens = 25 + (user_prompt.len() / 4) as i64;
        let completion_tokens = completion.len() / 4 as i64;

        llm_span.set_attribute(KeyValue::new("gen_ai.usage.prompt_tokens", prompt_tokens));
        llm_span.set_attribute(KeyValue::new(
            "gen_ai.usage.completion_tokens",
            completion_tokens,
        ));
        llm_span.set_attribute(KeyValue::new(
            "gen_ai.usage.total_tokens",
            prompt_tokens + completion_tokens,
        ));

        llm_span.set_attribute(KeyValue::new(
            "langfuse.observation.usage.input",
            prompt_tokens,
        ));
        llm_span.set_attribute(KeyValue::new(
            "langfuse.observation.usage.output",
            completion_tokens,
        ));
        llm_span.set_attribute(KeyValue::new(
            "langfuse.observation.usage.total",
            prompt_tokens + completion_tokens,
        ));

        llm_span.end();

        println!("LLM Response: {}", completion);
        println!(
            "Tokens used - Prompt: {}, Completion: {}, Total: {}",
            prompt_tokens,
            completion_tokens,
            prompt_tokens + completion_tokens
        );

        conversation_span.set_attribute(KeyValue::new("assistant.output", completion));
        conversation_span.set_attribute(KeyValue::new("conversation.success", true));

        conversation_span.end();

        sleep(Duration::from_millis(100)).await;
    }

    println!("\n--- Demo Complete ---");
    println!("Traces have been sent to Langfuse.");
    println!("Check your Langfuse dashboard to view the traces.");

    sleep(Duration::from_secs(2)).await;

    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
