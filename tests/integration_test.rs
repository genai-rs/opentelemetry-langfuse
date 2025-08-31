//! Integration tests for OpenTelemetry-Langfuse library.

use opentelemetry::trace::{Span, SpanKind, Tracer};
use opentelemetry::KeyValue;
use opentelemetry_langfuse::{
    AttributeMapper, GenAIAttributeMapper, LangfuseAttributes, LangfuseTracerBuilder,
    OpenTelemetryGenAIAttributes, TracingContext, TracingContextBuilder,
};
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_sdk::testing::trace::NoopSpanExporter;
use serde_json::json;

#[tokio::test]
async fn test_context_inheritance() {
    let parent = TracingContext::new()
        .with_session("session-parent")
        .with_user("user-parent")
        .with_metadata("level", json!("parent"));

    let child = parent
        .child()
        .with_metadata("level", json!("child"))
        .with_metadata("child_only", json!("data"));

    assert_eq!(
        child.get_attribute("session.id"),
        Some(json!("session-parent"))
    );

    assert_eq!(
        child.get_attribute("langfuse.trace.metadata.level"),
        Some(json!("child"))
    );

    assert!(parent
        .get_attribute("langfuse.trace.metadata.child_only")
        .is_none());
}

#[tokio::test]
async fn test_attribute_mapping() {
    let mapper = GenAIAttributeMapper::new();

    let otel_attrs = vec![
        KeyValue::new(OpenTelemetryGenAIAttributes::REQUEST_MODEL, "gpt-4"),
        KeyValue::new(OpenTelemetryGenAIAttributes::USAGE_PROMPT_TOKENS, 100i64),
        KeyValue::new(
            OpenTelemetryGenAIAttributes::USAGE_COMPLETION_TOKENS,
            200i64,
        ),
    ];

    let langfuse_attrs = mapper.map_to_langfuse(&otel_attrs);

    assert!(langfuse_attrs
        .iter()
        .any(|kv| kv.key.as_str() == LangfuseAttributes::OBSERVATION_MODEL));

    assert!(langfuse_attrs
        .iter()
        .any(|kv| kv.key.as_str() == LangfuseAttributes::OBSERVATION_USAGE_INPUT));
    assert!(langfuse_attrs
        .iter()
        .any(|kv| kv.key.as_str() == LangfuseAttributes::OBSERVATION_USAGE_OUTPUT));

    let enriched = mapper.enrich_attributes(&otel_attrs);

    assert!(enriched
        .iter()
        .any(|kv| kv.key.as_str() == OpenTelemetryGenAIAttributes::USAGE_TOTAL_TOKENS));
}

#[tokio::test]
async fn test_tracer_builder_with_context() {
    let context = TracingContextBuilder::new()
        .session_id("test-session")
        .user_id("test-user")
        .model("gpt-4")
        .temperature(0.5)
        .build();

    let _tracer = LangfuseTracerBuilder::new(Tokio)
        .with_endpoint("http://localhost:4318/v1/traces")
        .with_service_name("test-service")
        .with_context(context)
        .build();

}

#[tokio::test]
async fn test_context_to_otel_attributes() {
    let context = TracingContext::new()
        .with_session("session-123")
        .with_user("user-456")
        .with_name("test-trace")
        .with_tags(vec!["tag1".to_string(), "tag2".to_string()])
        .with_model("gpt-4")
        .with_temperature(0.7)
        .with_max_tokens(1000);

    let attrs = context.to_otel_attributes();

    assert!(attrs.iter().any(|kv| kv.key.as_str() == "session.id"));
    assert!(attrs.iter().any(|kv| kv.key.as_str() == "user.id"));
    assert!(attrs
        .iter()
        .any(|kv| kv.key.as_str() == LangfuseAttributes::TRACE_NAME));
    assert!(attrs
        .iter()
        .any(|kv| kv.key.as_str() == LangfuseAttributes::TRACE_TAGS));
    assert!(attrs
        .iter()
        .any(|kv| kv.key.as_str() == OpenTelemetryGenAIAttributes::REQUEST_MODEL));
    assert!(attrs
        .iter()
        .any(|kv| kv.key.as_str() == OpenTelemetryGenAIAttributes::REQUEST_TEMPERATURE));
    assert!(attrs
        .iter()
        .any(|kv| kv.key.as_str() == OpenTelemetryGenAIAttributes::REQUEST_MAX_TOKENS));
}

#[tokio::test]
async fn test_model_parameters_aggregation() {
    let mapper = GenAIAttributeMapper::new();

    let otel_attrs = vec![
        KeyValue::new(OpenTelemetryGenAIAttributes::REQUEST_TEMPERATURE, 0.7),
        KeyValue::new(OpenTelemetryGenAIAttributes::REQUEST_MAX_TOKENS, 1000i64),
        KeyValue::new(OpenTelemetryGenAIAttributes::REQUEST_TOP_P, 0.9),
        KeyValue::new(OpenTelemetryGenAIAttributes::REQUEST_FREQUENCY_PENALTY, 0.5),
    ];

    let langfuse_attrs = mapper.map_to_langfuse(&otel_attrs);

    assert!(langfuse_attrs
        .iter()
        .any(|kv| kv.key.as_str() == LangfuseAttributes::OBSERVATION_MODEL_PARAMETERS));
}

#[tokio::test]
async fn test_bidirectional_mapping() {
    let mapper = GenAIAttributeMapper::new();

    let original = vec![
        KeyValue::new("user.id", "user-123"),
        KeyValue::new(OpenTelemetryGenAIAttributes::REQUEST_MODEL, "gpt-4"),
    ];

    let langfuse = mapper.map_to_langfuse(&original);

    let back_to_otel = mapper.map_to_otel(&langfuse);

    assert!(back_to_otel.iter().any(|kv| kv.key.as_str() == "user.id"));
    assert!(back_to_otel
        .iter()
        .any(|kv| kv.key.as_str() == OpenTelemetryGenAIAttributes::REQUEST_MODEL));
}

#[tokio::test]
async fn test_context_clear() {
    let context = TracingContext::new()
        .with_session("session-123")
        .with_user("user-456");

    assert!(!context.get_all_attributes().is_empty());

    context.clear();

    assert!(context.get_all_attributes().is_empty());
}

#[tokio::test]
async fn test_context_merge() {
    let context1 = TracingContext::new()
        .with_session("session-1")
        .with_user("user-1");

    let context2 = TracingContext::new()
        .with_user("user-2")  // Should overwrite
        .with_model("gpt-4"); // New attribute

    context1.merge(&context2);

    assert_eq!(context1.get_attribute("user.id"), Some(json!("user-2")));

    assert_eq!(
        context1.get_attribute(OpenTelemetryGenAIAttributes::REQUEST_MODEL),
        Some(json!("gpt-4"))
    );

    assert_eq!(
        context1.get_attribute("session.id"),
        Some(json!("session-1"))
    );
}
