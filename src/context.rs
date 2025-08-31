//! Context management for Langfuse traces.
//!
//! This module provides a thread-safe context for managing trace attributes
//! that can be passed explicitly through your application.

use opentelemetry::trace::SpanContext;
use opentelemetry::KeyValue;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::attributes::{LangfuseAttributes, OpenTelemetryGenAIAttributes};

/// Thread-safe context for managing trace attributes.
#[derive(Clone, Debug)]
pub struct TracingContext {
    attributes: Arc<RwLock<HashMap<String, JsonValue>>>,
    parent_span: Option<SpanContext>,
}

impl TracingContext {
    /// Creates a new empty context.
    pub fn new() -> Self {
        Self {
            attributes: Arc::new(RwLock::new(HashMap::new())),
            parent_span: None,
        }
    }

    /// Creates a child context that inherits attributes from this context.
    pub fn child(&self) -> Self {
        let parent_attrs = self.attributes.read().unwrap().clone();
        Self {
            attributes: Arc::new(RwLock::new(parent_attrs)),
            parent_span: self.parent_span.clone(),
        }
    }

    /// Sets an attribute in the context.
    pub fn set_attribute(&self, key: impl Into<String>, value: JsonValue) {
        let mut attrs = self.attributes.write().unwrap();
        attrs.insert(key.into(), value);
    }

    /// Gets an attribute from the context.
    pub fn get_attribute(&self, key: impl AsRef<str>) -> Option<JsonValue> {
        let attrs = self.attributes.read().unwrap();
        attrs.get(key.as_ref()).cloned()
    }

    /// Gets all attributes from the context.
    pub fn get_all_attributes(&self) -> HashMap<String, JsonValue> {
        self.attributes.read().unwrap().clone()
    }

    /// Converts context attributes to OpenTelemetry KeyValue pairs.
    pub fn to_otel_attributes(&self) -> Vec<KeyValue> {
        let attrs = self.attributes.read().unwrap();
        attrs
            .iter()
            .map(|(k, v)| {
                let value = match v {
                    JsonValue::String(s) => opentelemetry::Value::from(s.clone()),
                    JsonValue::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            opentelemetry::Value::from(i)
                        } else if let Some(f) = n.as_f64() {
                            opentelemetry::Value::from(f)
                        } else {
                            opentelemetry::Value::from(n.to_string())
                        }
                    }
                    JsonValue::Bool(b) => opentelemetry::Value::from(*b),
                    _ => opentelemetry::Value::from(v.to_string()),
                };
                KeyValue::new(k.clone(), value)
            })
            .collect()
    }

    /// Merges attributes from another context into this one.
    pub fn merge(&self, other: &TracingContext) {
        let mut attrs = self.attributes.write().unwrap();
        let other_attrs = other.attributes.read().unwrap();
        for (k, v) in other_attrs.iter() {
            attrs.insert(k.clone(), v.clone());
        }
    }

    /// Clears all attributes from the context.
    pub fn clear(&self) {
        let mut attrs = self.attributes.write().unwrap();
        attrs.clear();
    }

    /// Builder-style method to set the trace name.
    pub fn with_name(self, name: impl Into<String>) -> Self {
        self.set_attribute(
            LangfuseAttributes::TRACE_NAME,
            JsonValue::String(name.into()),
        );
        self
    }

    /// Builder-style method to set the user ID.
    pub fn with_user(self, user_id: impl Into<String>) -> Self {
        self.set_attribute(
            LangfuseAttributes::TRACE_USER_ID,
            JsonValue::String(user_id.into()),
        );
        self
    }

    /// Builder-style method to set the session ID.
    pub fn with_session(self, session_id: impl Into<String>) -> Self {
        self.set_attribute(
            LangfuseAttributes::TRACE_SESSION_ID,
            JsonValue::String(session_id.into()),
        );
        self
    }

    /// Builder-style method to set tags.
    pub fn with_tags(self, tags: Vec<String>) -> Self {
        self.set_attribute(
            LangfuseAttributes::TRACE_TAGS,
            JsonValue::Array(tags.into_iter().map(JsonValue::String).collect()),
        );
        self
    }

    /// Builder-style method to add metadata.
    pub fn with_metadata(self, key: impl Into<String>, value: JsonValue) -> Self {
        let metadata_key = format!("{}.{}", LangfuseAttributes::TRACE_METADATA, key.into());
        self.set_attribute(metadata_key, value);
        self
    }

    /// Builder-style method to set the model.
    pub fn with_model(self, model: impl Into<String>) -> Self {
        self.set_attribute(
            OpenTelemetryGenAIAttributes::REQUEST_MODEL,
            JsonValue::String(model.into()),
        );
        self
    }

    /// Builder-style method to set temperature.
    pub fn with_temperature(self, temperature: f64) -> Self {
        self.set_attribute(
            OpenTelemetryGenAIAttributes::REQUEST_TEMPERATURE,
            serde_json::json!(temperature),
        );
        self
    }

    /// Builder-style method to set max tokens.
    pub fn with_max_tokens(self, max_tokens: i64) -> Self {
        self.set_attribute(
            OpenTelemetryGenAIAttributes::REQUEST_MAX_TOKENS,
            serde_json::json!(max_tokens),
        );
        self
    }
}

impl Default for TracingContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating TracingContext instances.
#[derive(Default)]
pub struct TracingContextBuilder {
    attributes: HashMap<String, JsonValue>,
}

impl TracingContextBuilder {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the session ID.
    pub fn session_id(mut self, id: impl Into<String>) -> Self {
        self.attributes.insert(
            LangfuseAttributes::TRACE_SESSION_ID.to_string(),
            JsonValue::String(id.into()),
        );
        self
    }

    /// Sets the user ID.
    pub fn user_id(mut self, id: impl Into<String>) -> Self {
        self.attributes.insert(
            LangfuseAttributes::TRACE_USER_ID.to_string(),
            JsonValue::String(id.into()),
        );
        self
    }

    /// Sets the model.
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.attributes.insert(
            OpenTelemetryGenAIAttributes::REQUEST_MODEL.to_string(),
            JsonValue::String(model.into()),
        );
        self
    }

    /// Sets the temperature.
    pub fn temperature(mut self, temp: f64) -> Self {
        self.attributes.insert(
            OpenTelemetryGenAIAttributes::REQUEST_TEMPERATURE.to_string(),
            serde_json::json!(temp),
        );
        self
    }

    /// Builds the TracingContext.
    pub fn build(self) -> TracingContext {
        let context = TracingContext::new();
        for (k, v) in self.attributes {
            context.set_attribute(k, v);
        }
        context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_attributes() {
        let context = TracingContext::new();
        context.set_attribute("key1", JsonValue::String("value1".to_string()));
        context.set_attribute("key2", serde_json::json!(42));

        assert_eq!(
            context.get_attribute("key1"),
            Some(JsonValue::String("value1".to_string()))
        );
        assert_eq!(context.get_attribute("key2"), Some(serde_json::json!(42)));
        assert_eq!(context.get_attribute("key3"), None);
    }

    #[test]
    fn test_context_child() {
        let parent = TracingContext::new()
            .with_session("session-parent")
            .with_user("user-parent");

        let child = parent.child().with_name("child-trace");

        assert_eq!(
            child.get_attribute(LangfuseAttributes::TRACE_SESSION_ID),
            Some(JsonValue::String("session-parent".to_string()))
        );

        assert_eq!(
            child.get_attribute(LangfuseAttributes::TRACE_NAME),
            Some(JsonValue::String("child-trace".to_string()))
        );

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

        assert!(otel_attrs
            .iter()
            .any(|kv| kv.key.as_str() == LangfuseAttributes::TRACE_SESSION_ID));
        assert!(otel_attrs
            .iter()
            .any(|kv| kv.key.as_str() == OpenTelemetryGenAIAttributes::REQUEST_TEMPERATURE));
    }
}
