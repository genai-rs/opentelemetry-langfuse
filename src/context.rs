//! Context management for OpenTelemetry-Langfuse integration.
//!
//! This module provides a thread-safe, non-global approach to managing trace attributes
//! and context, allowing explicit context passing through the application.

use crate::attributes::{LangfuseAttributes, OpenTelemetryGenAIAttributes};
use opentelemetry::{
    trace::{SpanContext, TraceContextExt},
    Context as OtelContext, KeyValue,
};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Thread-safe context for managing trace attributes.
///
/// This struct provides a way to pass context through your application
/// without relying on global state, making testing and dependency
/// injection much cleaner.
#[derive(Clone, Debug)]
pub struct TracingContext {
    attributes: Arc<RwLock<HashMap<String, JsonValue>>>,
    parent_span: Option<SpanContext>,
}

impl TracingContext {
    /// Creates a new empty tracing context.
    pub fn new() -> Self {
        Self {
            attributes: Arc::new(RwLock::new(HashMap::new())),
            parent_span: None,
        }
    }

    /// Creates a new context from an OpenTelemetry context.
    pub fn from_otel_context(ctx: &OtelContext) -> Self {
        let parent_span = ctx.span().span_context().clone();
        Self {
            attributes: Arc::new(RwLock::new(HashMap::new())),
            parent_span: Some(parent_span),
        }
    }

    /// Sets a single attribute in the context.
    pub fn set_attribute(&self, key: impl Into<String>, value: impl Into<JsonValue>) -> &Self {
        if let Ok(mut attrs) = self.attributes.write() {
            attrs.insert(key.into(), value.into());
        }
        self
    }

    /// Sets multiple attributes at once.
    pub fn set_attributes(&self, attributes: HashMap<String, JsonValue>) -> &Self {
        if let Ok(mut attrs) = self.attributes.write() {
            attrs.extend(attributes);
        }
        self
    }

    /// Gets an attribute value by key.
    pub fn get_attribute(&self, key: &str) -> Option<JsonValue> {
        self.attributes.read().ok()?.get(key).cloned()
    }

    /// Gets all attributes as a HashMap.
    pub fn get_all_attributes(&self) -> HashMap<String, JsonValue> {
        self.attributes
            .read()
            .map(|attrs| attrs.clone())
            .unwrap_or_default()
    }

    /// Converts attributes to OpenTelemetry KeyValue pairs.
    pub fn to_otel_attributes(&self) -> Vec<KeyValue> {
        self.attributes
            .read()
            .map(|attrs| {
                attrs
                    .iter()
                    .map(|(k, v)| self.json_to_key_value(k.clone(), v.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Sets the session ID for the trace.
    pub fn with_session(self, session_id: impl Into<String>) -> Self {
        self.set_attribute(LangfuseAttributes::TRACE_SESSION_ID, session_id.into());
        self
    }

    /// Sets the user ID for the trace.
    pub fn with_user(self, user_id: impl Into<String>) -> Self {
        self.set_attribute(LangfuseAttributes::TRACE_USER_ID, user_id.into());
        self
    }

    /// Sets the trace name.
    pub fn with_name(self, name: impl Into<String>) -> Self {
        self.set_attribute(LangfuseAttributes::TRACE_NAME, name.into());
        self
    }

    /// Adds tags to the trace.
    pub fn with_tags(self, tags: Vec<String>) -> Self {
        self.set_attribute(
            LangfuseAttributes::TRACE_TAGS,
            JsonValue::Array(tags.into_iter().map(JsonValue::String).collect()),
        );
        self
    }

    /// Adds metadata to the trace.
    pub fn with_metadata(self, key: impl Into<String>, value: JsonValue) -> Self {
        let metadata_key = format!("{}.{}", LangfuseAttributes::TRACE_METADATA, key.into());
        self.set_attribute(metadata_key, value);
        self
    }

    /// Sets the GenAI model being used.
    pub fn with_model(self, model: impl Into<String>) -> Self {
        self.set_attribute(OpenTelemetryGenAIAttributes::REQUEST_MODEL, model.into());
        self
    }

    /// Sets the temperature parameter for the model.
    pub fn with_temperature(self, temperature: f64) -> Self {
        self.set_attribute(
            OpenTelemetryGenAIAttributes::REQUEST_TEMPERATURE,
            JsonValue::Number(
                serde_json::Number::from_f64(temperature).unwrap_or_else(|| 0.into()),
            ),
        );
        self
    }

    /// Sets the maximum tokens parameter.
    pub fn with_max_tokens(self, max_tokens: i64) -> Self {
        self.set_attribute(
            OpenTelemetryGenAIAttributes::REQUEST_MAX_TOKENS,
            JsonValue::Number(max_tokens.into()),
        );
        self
    }

    /// Gets the parent span context if available.
    pub fn parent_span(&self) -> Option<&SpanContext> {
        self.parent_span.as_ref()
    }

    /// Merges another context into this one.
    ///
    /// Attributes from the other context will overwrite existing ones
    /// with the same key.
    pub fn merge(&self, other: &TracingContext) -> &Self {
        if let (Ok(mut attrs), Ok(other_attrs)) = (self.attributes.write(), other.attributes.read())
        {
            attrs.extend(other_attrs.clone());
        }
        self
    }

    /// Creates a child context that inherits attributes from this one.
    pub fn child(&self) -> Self {
        let mut child = Self::new();
        if let Ok(attrs) = self.attributes.read() {
            if let Ok(mut child_attrs) = child.attributes.write() {
                *child_attrs = attrs.clone();
            }
        }
        child.parent_span = self.parent_span.clone();
        child
    }

    /// Clears all attributes from the context.
    pub fn clear(&self) -> &Self {
        if let Ok(mut attrs) = self.attributes.write() {
            attrs.clear();
        }
        self
    }

    fn json_to_key_value(&self, key: String, value: JsonValue) -> KeyValue {
        match value {
            JsonValue::String(s) => KeyValue::new(key, s),
            JsonValue::Bool(b) => KeyValue::new(key, b),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    KeyValue::new(key, i)
                } else if let Some(f) = n.as_f64() {
                    KeyValue::new(key, f)
                } else {
                    KeyValue::new(key, n.to_string())
                }
            }
            _ => KeyValue::new(key, value.to_string()),
        }
    }
}

impl Default for TracingContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating TracingContext with a fluent API.
pub struct TracingContextBuilder {
    context: TracingContext,
}

impl TracingContextBuilder {
    /// Creates a new context builder.
    pub fn new() -> Self {
        Self {
            context: TracingContext::new(),
        }
    }

    /// Sets the session ID.
    pub fn session_id(self, session_id: impl Into<String>) -> Self {
        self.context
            .set_attribute(LangfuseAttributes::TRACE_SESSION_ID, session_id.into());
        self
    }

    /// Sets the user ID.
    pub fn user_id(self, user_id: impl Into<String>) -> Self {
        self.context
            .set_attribute(LangfuseAttributes::TRACE_USER_ID, user_id.into());
        self
    }

    /// Sets the trace name.
    pub fn name(self, name: impl Into<String>) -> Self {
        self.context
            .set_attribute(LangfuseAttributes::TRACE_NAME, name.into());
        self
    }

    /// Adds tags to the trace.
    pub fn tags(self, tags: Vec<String>) -> Self {
        self.context.set_attribute(
            LangfuseAttributes::TRACE_TAGS,
            JsonValue::Array(tags.into_iter().map(JsonValue::String).collect()),
        );
        self
    }

    /// Sets whether the trace is public.
    pub fn public(self, is_public: bool) -> Self {
        self.context
            .set_attribute(LangfuseAttributes::TRACE_PUBLIC, JsonValue::Bool(is_public));
        self
    }

    /// Adds metadata.
    pub fn metadata(self, key: impl Into<String>, value: JsonValue) -> Self {
        let metadata_key = format!("{}.{}", LangfuseAttributes::TRACE_METADATA, key.into());
        self.context.set_attribute(metadata_key, value);
        self
    }

    /// Sets the model.
    pub fn model(self, model: impl Into<String>) -> Self {
        self.context
            .set_attribute(OpenTelemetryGenAIAttributes::REQUEST_MODEL, model.into());
        self
    }

    /// Sets the temperature.
    pub fn temperature(self, temperature: f64) -> Self {
        self.context.set_attribute(
            OpenTelemetryGenAIAttributes::REQUEST_TEMPERATURE,
            JsonValue::Number(
                serde_json::Number::from_f64(temperature).unwrap_or_else(|| 0.into()),
            ),
        );
        self
    }

    /// Sets the maximum tokens.
    pub fn max_tokens(self, max_tokens: i64) -> Self {
        self.context.set_attribute(
            OpenTelemetryGenAIAttributes::REQUEST_MAX_TOKENS,
            JsonValue::Number(max_tokens.into()),
        );
        self
    }

    /// Sets a custom attribute.
    pub fn attribute(self, key: impl Into<String>, value: impl Into<JsonValue>) -> Self {
        self.context.set_attribute(key, value);
        self
    }

    /// Builds the TracingContext.
    pub fn build(self) -> TracingContext {
        self.context
    }
}

impl Default for TracingContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_context_creation() {
        let context = TracingContext::new();
        assert!(context.get_all_attributes().is_empty());
    }

    #[test]
    fn test_context_with_attributes() {
        let context = TracingContext::new()
            .with_session("session-123")
            .with_user("user-456")
            .with_name("test-trace");

        assert_eq!(
            context.get_attribute(LangfuseAttributes::TRACE_SESSION_ID),
            Some(JsonValue::String("session-123".to_string()))
        );
        assert_eq!(
            context.get_attribute(LangfuseAttributes::TRACE_USER_ID),
            Some(JsonValue::String("user-456".to_string()))
        );
    }

    #[test]
    fn test_context_builder() {
        let context = TracingContextBuilder::new()
            .session_id("session-789")
            .user_id("user-012")
            .model("gpt-4")
            .temperature(0.7)
            .build();

        let attrs = context.get_all_attributes();
        assert_eq!(attrs.len(), 4);
        assert_eq!(
            context.get_attribute(LangfuseAttributes::TRACE_SESSION_ID),
            Some(JsonValue::String("session-789".to_string()))
        );
    }

    #[test]
    fn test_context_merge() {
        let context1 = TracingContext::new()
            .with_session("session-1")
            .with_user("user-1");

        let context2 = TracingContext::new()
            .with_user("user-2")  // Should overwrite
            .with_model("gpt-4");

        context1.merge(&context2);

        assert_eq!(
            context1.get_attribute(LangfuseAttributes::TRACE_USER_ID),
            Some(JsonValue::String("user-2".to_string()))
        );
        assert_eq!(
            context1.get_attribute(OpenTelemetryGenAIAttributes::REQUEST_MODEL),
            Some(JsonValue::String("gpt-4".to_string()))
        );
    }

    #[test]
    fn test_context_child() {
        let parent = TracingContext::new()
            .with_session("session-parent")
            .with_user("user-parent");

        let child = parent.child().with_name("child-trace");

        // Child should inherit parent attributes
        assert_eq!(
            child.get_attribute(LangfuseAttributes::TRACE_SESSION_ID),
            Some(JsonValue::String("session-parent".to_string()))
        );

        // Child should have its own attributes
        assert_eq!(
            child.get_attribute(LangfuseAttributes::TRACE_NAME),
            Some(JsonValue::String("child-trace".to_string()))
        );

        // Parent should not have child's attributes
        assert_eq!(parent.get_attribute(LangfuseAttributes::TRACE_NAME), None);
    }

    #[test]
    fn test_to_otel_attributes() {
        let context = TracingContext::new()
            .with_session("session-123")
            .with_temperature(0.5)
            .with_max_tokens(1000);

        let otel_attrs = context.to_otel_attributes();
        assert_eq!(otel_attrs.len(), 3);

        // Check that attributes are properly converted
        assert!(otel_attrs
            .iter()
            .any(|kv| kv.key.as_str() == LangfuseAttributes::TRACE_SESSION_ID));
        assert!(otel_attrs
            .iter()
            .any(|kv| kv.key.as_str() == OpenTelemetryGenAIAttributes::REQUEST_TEMPERATURE));
    }

    #[test]
    fn test_context_with_metadata() {
        let context = TracingContext::new()
            .with_metadata("environment", json!("production"))
            .with_metadata("version", json!("1.0.0"));

        let attrs = context.get_all_attributes();
        assert_eq!(attrs.len(), 2);

        let env_key = format!("{}.environment", LangfuseAttributes::TRACE_METADATA);
        assert_eq!(
            context.get_attribute(&env_key),
            Some(JsonValue::String("production".to_string()))
        );
    }
}
