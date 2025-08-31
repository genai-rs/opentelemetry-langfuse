//! Example demonstrating how to enrich OpenTelemetry spans with Langfuse attributes

use opentelemetry::global;
use opentelemetry::trace::{Span, SpanKind, TraceContextExt, Tracer, TracerProvider};
use opentelemetry::{Context, KeyValue};
use opentelemetry_langfuse::{exporter_from_env, GenAISpanExt, LangfuseSpanExt};
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;
use serde_json::json;
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
                    KeyValue::new("service.name", "span-enrichment-example"),
                    KeyValue::new("service.version", "0.1.0"),
                ])
                .build(),
        )
        .build();

    let tracer = provider.tracer("example-tracer");

    global::set_tracer_provider(provider.clone());

    println!("=== Span Enrichment Example ===\n");

    example_basic_enrichment(&tracer).await?;
    example_llm_generation(&tracer).await?;
    example_nested_spans(&tracer).await?;

    println!("\n=== Example Complete ===");

    sleep(Duration::from_secs(2)).await;
    provider.shutdown()?;

    Ok(())
}

async fn example_basic_enrichment<T: Tracer>(tracer: &T) -> Result<(), Box<dyn Error>> {
    println!("Example 1: Basic span enrichment");

    let mut span = tracer
        .span_builder("user_request")
        .with_kind(SpanKind::Server)
        .start(tracer);

    span.set_user_id("user-123")
        .set_session_id("session-456")
        .set_trace_name("User Authentication Flow")
        .set_trace_tags(vec!["auth".to_string(), "login".to_string()])
        .add_langfuse_metadata("ip_address", json!("192.168.1.1"))
        .add_langfuse_metadata("user_agent", json!("Mozilla/5.0"));

    sleep(Duration::from_millis(100)).await;

    span.set_status(opentelemetry::trace::Status::Ok);
    span.end();

    println!("  ✓ Enriched span with user context and metadata\n");
    Ok(())
}

async fn example_llm_generation<T: Tracer>(tracer: &T) -> Result<(), Box<dyn Error>> {
    println!("Example 2: LLM generation with GenAI attributes");

    let mut span = tracer
        .span_builder("llm_generation")
        .with_kind(SpanKind::Client)
        .start(tracer);

    span.set_observation_type("generation")
        .set_observation_model("gpt-4")
        .set_gen_ai_temperature(0.7)
        .set_gen_ai_max_tokens(1000)
        .set_gen_ai_top_p(0.9)
        .set_gen_ai_prompt(0, "system", "You are a helpful assistant")
        .set_gen_ai_prompt(1, "user", "What is OpenTelemetry?");

    sleep(Duration::from_millis(200)).await;

    let response = "OpenTelemetry is an observability framework for cloud-native software...";

    span.set_gen_ai_completion(0, "assistant", response)
        .set_gen_ai_usage(50, 150)
        .set_model_parameters(json!({
            "temperature": 0.7,
            "max_tokens": 1000,
            "top_p": 0.9,
            "frequency_penalty": 0.0,
            "presence_penalty": 0.0
        }));

    span.end();

    println!("  ✓ Enriched span with GenAI semantic conventions");
    println!("    - Model: gpt-4");
    println!("    - Tokens: 50 prompt + 150 completion = 200 total\n");

    Ok(())
}

async fn example_nested_spans<T>(tracer: &T) -> Result<(), Box<dyn Error>>
where
    T: Tracer,
    T::Span: Send + Sync + 'static,
{
    println!("Example 3: Nested spans with shared context");

    let mut parent_span = tracer
        .span_builder("chat_conversation")
        .with_kind(SpanKind::Server)
        .start(tracer);

    parent_span
        .set_user_id("user-789")
        .set_session_id("session-abc")
        .set_trace_name("Customer Support Chat")
        .add_langfuse_metadata("channel", json!("web"));

    let cx = Context::current_with_span(parent_span);
    let _guard = cx.attach();

    for i in 1..=3 {
        let mut child_span = tracer
            .span_builder(format!("message_{}", i))
            .with_kind(SpanKind::Internal)
            .start(tracer);

        child_span
            .set_observation_type("generation")
            .add_langfuse_metadata("message_index", json!(i))
            .set_gen_ai_prompt(0, "user", format!("User message {}", i))
            .set_gen_ai_completion(0, "assistant", format!("Response to message {}", i))
            .set_gen_ai_usage(10 * i as i64, 20 * i as i64);

        sleep(Duration::from_millis(50)).await;
        child_span.end();
    }

    println!("  ✓ Created nested spans with Langfuse attributes");
    println!("    - Parent: chat_conversation");
    println!("    - Children: 3 message spans\n");

    Ok(())
}
