use opentelemetry::trace::noop::NoopTracer;
use opentelemetry::trace::{Span, Tracer};
use opentelemetry_langfuse::{GenAISpanExt, LangfuseSpanExt};
use serde_json::json;

#[test]
fn test_langfuse_span_extensions() {
    let tracer = NoopTracer::new();
    let mut span = tracer.span_builder("test").start(&tracer);

    span.set_user_id("user-123")
        .set_session_id("session-456")
        .set_trace_name("Test Trace")
        .set_trace_tags(vec!["test".to_string(), "example".to_string()])
        .set_observation_type("generation")
        .set_observation_model("gpt-4");

    span.end();
}

#[test]
fn test_genai_span_extensions() {
    let tracer = NoopTracer::new();
    let mut span = tracer.span_builder("test").start(&tracer);

    span.set_gen_ai_model("gpt-4")
        .set_gen_ai_temperature(0.7)
        .set_gen_ai_max_tokens(1000)
        .set_gen_ai_top_p(0.9)
        .set_gen_ai_usage(100, 200);

    span.end();
}

#[test]
fn test_model_parameters() {
    let tracer = NoopTracer::new();
    let mut span = tracer.span_builder("test").start(&tracer);

    let params = json!({
        "temperature": 0.7,
        "max_tokens": 1000,
        "top_p": 0.9,
        "frequency_penalty": 0.5,
        "presence_penalty": 0.3
    });

    span.set_model_parameters(params);
    span.end();
}

#[test]
fn test_prompt_and_completion() {
    let tracer = NoopTracer::new();
    let mut span = tracer.span_builder("test").start(&tracer);

    span.set_gen_ai_prompt(0, "system", "You are a helpful assistant")
        .set_gen_ai_prompt(1, "user", "Hello!")
        .set_gen_ai_completion(0, "assistant", "Hi there! How can I help you?");

    span.end();
}

#[test]
fn test_langfuse_metadata() {
    let tracer = NoopTracer::new();
    let mut span = tracer.span_builder("test").start(&tracer);

    span.add_langfuse_metadata("custom_field", json!("custom_value"))
        .add_langfuse_metadata("priority", json!(1))
        .add_langfuse_metadata("tags", json!(["tag1", "tag2"]));

    span.end();
}

#[test]
fn test_token_usage() {
    let tracer = NoopTracer::new();
    let mut span = tracer.span_builder("test").start(&tracer);

    span.set_input_tokens(100)
        .set_output_tokens(200)
        .set_total_tokens(300);

    span.end();
}

#[test]
fn test_chained_enrichment() {
    let tracer = NoopTracer::new();
    let mut span = tracer.span_builder("test").start(&tracer);

    span.set_user_id("user-123")
        .set_session_id("session-456")
        .set_observation_model("gpt-4")
        .set_gen_ai_temperature(0.7)
        .set_gen_ai_usage(150, 250)
        .add_langfuse_metadata("environment", json!("production"));

    span.end();
}
