//! Attribute mapping between Langfuse and OpenTelemetry GenAI conventions.
//!
//! This module provides bidirectional mapping capabilities between Langfuse-specific
//! attributes and OpenTelemetry GenAI semantic conventions.

use crate::attributes::{LangfuseAttributes, OpenTelemetryGenAIAttributes};
use opentelemetry::KeyValue;
use serde_json::{json, Value as JsonValue};
use std::collections::HashMap;

/// Trait for mapping attributes between different conventions.
pub trait AttributeMapper: Send + Sync {
    /// Maps OpenTelemetry attributes to Langfuse attributes.
    fn map_to_langfuse(&self, otel_attributes: &[KeyValue]) -> Vec<KeyValue>;

    /// Maps Langfuse attributes to OpenTelemetry attributes.
    fn map_to_otel(&self, langfuse_attributes: &[KeyValue]) -> Vec<KeyValue>;

    /// Enriches attributes with additional mappings.
    fn enrich_attributes(&self, attributes: &[KeyValue]) -> Vec<KeyValue>;
}

/// Defines how an attribute should be mapped.
#[derive(Clone, Debug)]
pub enum MappingRule {
    /// Direct bidirectional mapping between attribute names.
    Bidirectional {
        langfuse_key: String,
        otel_key: String,
    },
    /// Transform the value during mapping.
    Transform {
        source_key: String,
        target_key: String,
        transformer: fn(&JsonValue) -> JsonValue,
    },
    /// One-way mapping from source to target.
    OneWay {
        source_key: String,
        target_key: String,
    },
    /// Complex mapping that may produce multiple attributes.
    Complex {
        source_key: String,
        mapper: fn(&str, &JsonValue) -> Vec<(String, JsonValue)>,
    },
}

/// Mapper for GenAI-specific attributes.
pub struct GenAIAttributeMapper {
    mapping_rules: HashMap<String, MappingRule>,
}

impl GenAIAttributeMapper {
    /// Creates a new GenAI attribute mapper with default mappings.
    pub fn new() -> Self {
        let mut rules = HashMap::new();

        // Model mappings
        rules.insert(
            OpenTelemetryGenAIAttributes::REQUEST_MODEL.to_string(),
            MappingRule::Bidirectional {
                langfuse_key: LangfuseAttributes::OBSERVATION_MODEL.to_string(),
                otel_key: OpenTelemetryGenAIAttributes::REQUEST_MODEL.to_string(),
            },
        );

        // Token usage mappings
        rules.insert(
            OpenTelemetryGenAIAttributes::USAGE_PROMPT_TOKENS.to_string(),
            MappingRule::Transform {
                source_key: OpenTelemetryGenAIAttributes::USAGE_PROMPT_TOKENS.to_string(),
                target_key: LangfuseAttributes::OBSERVATION_USAGE_INPUT.to_string(),
                transformer: |v| v.clone(),
            },
        );

        rules.insert(
            OpenTelemetryGenAIAttributes::USAGE_COMPLETION_TOKENS.to_string(),
            MappingRule::Transform {
                source_key: OpenTelemetryGenAIAttributes::USAGE_COMPLETION_TOKENS.to_string(),
                target_key: LangfuseAttributes::OBSERVATION_USAGE_OUTPUT.to_string(),
                transformer: |v| v.clone(),
            },
        );

        // Model parameters complex mapping
        rules.insert(
            "model_parameters".to_string(),
            MappingRule::Complex {
                source_key: "gen_ai.request.*".to_string(),
                mapper: |key, value| {
                    if key.starts_with("gen_ai.request.") && !key.contains("model") {
                        vec![(
                            LangfuseAttributes::OBSERVATION_MODEL_PARAMETERS.to_string(),
                            value.clone(),
                        )]
                    } else {
                        vec![]
                    }
                },
            },
        );

        // User ID mapping
        rules.insert(
            "user.id".to_string(),
            MappingRule::Bidirectional {
                langfuse_key: LangfuseAttributes::TRACE_USER_ID.to_string(),
                otel_key: "user.id".to_string(),
            },
        );

        // Session ID mapping
        rules.insert(
            "session.id".to_string(),
            MappingRule::Bidirectional {
                langfuse_key: LangfuseAttributes::TRACE_SESSION_ID.to_string(),
                otel_key: "session.id".to_string(),
            },
        );

        Self {
            mapping_rules: rules,
        }
    }

    /// Adds a custom mapping rule.
    pub fn add_rule(&mut self, key: String, rule: MappingRule) {
        self.mapping_rules.insert(key, rule);
    }

    /// Removes a mapping rule.
    pub fn remove_rule(&mut self, key: &str) -> Option<MappingRule> {
        self.mapping_rules.remove(key)
    }

    fn key_value_to_json(&self, kv: &KeyValue) -> JsonValue {
        match kv.value {
            opentelemetry::Value::Bool(b) => JsonValue::Bool(b),
            opentelemetry::Value::I64(i) => JsonValue::Number(i.into()),
            opentelemetry::Value::F64(f) => {
                JsonValue::Number(serde_json::Number::from_f64(f).unwrap_or_else(|| 0.into()))
            }
            opentelemetry::Value::String(ref s) => JsonValue::String(s.to_string()),
            opentelemetry::Value::Array(ref arr) => {
                let values: Vec<JsonValue> = arr
                    .iter()
                    .map(|v| match v {
                        opentelemetry::Value::Bool(b) => JsonValue::Bool(*b),
                        opentelemetry::Value::I64(i) => JsonValue::Number((*i).into()),
                        opentelemetry::Value::F64(f) => JsonValue::Number(
                            serde_json::Number::from_f64(*f).unwrap_or_else(|| 0.into()),
                        ),
                        opentelemetry::Value::String(s) => JsonValue::String(s.to_string()),
                        _ => JsonValue::Null,
                    })
                    .collect();
                JsonValue::Array(values)
            }
        }
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
            JsonValue::Array(arr) => {
                let string_array: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                KeyValue::new(key, string_array.join(","))
            }
            _ => KeyValue::new(key, value.to_string()),
        }
    }
}

impl Default for GenAIAttributeMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl AttributeMapper for GenAIAttributeMapper {
    fn map_to_langfuse(&self, otel_attributes: &[KeyValue]) -> Vec<KeyValue> {
        let mut result = Vec::new();
        let mut model_params = HashMap::new();

        for attr in otel_attributes {
            let key = attr.key.as_str();
            let value = self.key_value_to_json(attr);

            // Check if this attribute has a mapping rule
            if let Some(rule) = self.mapping_rules.get(key) {
                match rule {
                    MappingRule::Bidirectional { langfuse_key, .. } => {
                        result.push(self.json_to_key_value(langfuse_key.clone(), value));
                    }
                    MappingRule::Transform {
                        target_key,
                        transformer,
                        ..
                    } => {
                        let transformed = transformer(&value);
                        result.push(self.json_to_key_value(target_key.clone(), transformed));
                    }
                    MappingRule::OneWay { target_key, .. } => {
                        result.push(self.json_to_key_value(target_key.clone(), value));
                    }
                    MappingRule::Complex { mapper, .. } => {
                        let mapped = mapper(key, &value);
                        for (k, v) in mapped {
                            result.push(self.json_to_key_value(k, v));
                        }
                    }
                }
            } else if key.starts_with("gen_ai.request.") && !key.contains("model") {
                // Collect model parameters
                let param_name = key.strip_prefix("gen_ai.request.").unwrap_or(key);
                model_params.insert(param_name.to_string(), value);
            } else if key.starts_with("gen_ai.prompt.") || key.starts_with("gen_ai.completion.") {
                // Handle prompt and completion attributes
                if key.ends_with(".content") {
                    let target_key = if key.starts_with("gen_ai.prompt.") {
                        LangfuseAttributes::OBSERVATION_INPUT
                    } else {
                        LangfuseAttributes::OBSERVATION_OUTPUT
                    };
                    result.push(self.json_to_key_value(target_key.to_string(), value));
                }
            } else {
                // Pass through unmapped attributes
                result.push(attr.clone());
            }
        }

        // Add collected model parameters as a single JSON attribute
        if !model_params.is_empty() {
            result.push(self.json_to_key_value(
                LangfuseAttributes::OBSERVATION_MODEL_PARAMETERS.to_string(),
                json!(model_params),
            ));
        }

        result
    }

    fn map_to_otel(&self, langfuse_attributes: &[KeyValue]) -> Vec<KeyValue> {
        let mut result = Vec::new();

        for attr in langfuse_attributes {
            let key = attr.key.as_str();
            let value = self.key_value_to_json(attr);

            // Find reverse mapping
            let mut found_mapping = false;
            for (_, rule) in &self.mapping_rules {
                match rule {
                    MappingRule::Bidirectional {
                        langfuse_key,
                        otel_key,
                    } => {
                        if langfuse_key == key {
                            result.push(self.json_to_key_value(otel_key.clone(), value.clone()));
                            found_mapping = true;
                            break;
                        }
                    }
                    _ => {} // Other rule types are one-way from OTel to Langfuse
                }
            }

            // Special handling for certain Langfuse attributes
            if !found_mapping {
                match key {
                    k if k == LangfuseAttributes::OBSERVATION_USAGE_INPUT => {
                        result.push(self.json_to_key_value(
                            OpenTelemetryGenAIAttributes::USAGE_PROMPT_TOKENS.to_string(),
                            value,
                        ));
                    }
                    k if k == LangfuseAttributes::OBSERVATION_USAGE_OUTPUT => {
                        result.push(self.json_to_key_value(
                            OpenTelemetryGenAIAttributes::USAGE_COMPLETION_TOKENS.to_string(),
                            value,
                        ));
                    }
                    k if k == LangfuseAttributes::OBSERVATION_MODEL_PARAMETERS => {
                        // Expand model parameters back to individual gen_ai.request.* attributes
                        if let JsonValue::Object(params) = value {
                            for (param_key, param_value) in params {
                                let otel_key = format!("gen_ai.request.{}", param_key);
                                result.push(self.json_to_key_value(otel_key, param_value));
                            }
                        }
                    }
                    _ => {
                        // Pass through unmapped attributes
                        result.push(attr.clone());
                    }
                }
            }
        }

        result
    }

    fn enrich_attributes(&self, attributes: &[KeyValue]) -> Vec<KeyValue> {
        let mut enriched = attributes.to_vec();

        // Calculate total tokens if we have prompt and completion tokens
        let mut prompt_tokens = None;
        let mut completion_tokens = None;

        for attr in attributes {
            match attr.key.as_str() {
                k if k == OpenTelemetryGenAIAttributes::USAGE_PROMPT_TOKENS => {
                    if let opentelemetry::Value::I64(v) = attr.value {
                        prompt_tokens = Some(v);
                    }
                }
                k if k == OpenTelemetryGenAIAttributes::USAGE_COMPLETION_TOKENS => {
                    if let opentelemetry::Value::I64(v) = attr.value {
                        completion_tokens = Some(v);
                    }
                }
                _ => {}
            }
        }

        if let (Some(prompt), Some(completion)) = (prompt_tokens, completion_tokens) {
            enriched.push(KeyValue::new(
                OpenTelemetryGenAIAttributes::USAGE_TOTAL_TOKENS,
                prompt + completion,
            ));
            enriched.push(KeyValue::new(
                LangfuseAttributes::OBSERVATION_USAGE_TOTAL,
                prompt + completion,
            ));
        }

        enriched
    }
}

/// A pass-through mapper that doesn't modify attributes.
pub struct PassThroughMapper;

impl AttributeMapper for PassThroughMapper {
    fn map_to_langfuse(&self, otel_attributes: &[KeyValue]) -> Vec<KeyValue> {
        otel_attributes.to_vec()
    }

    fn map_to_otel(&self, langfuse_attributes: &[KeyValue]) -> Vec<KeyValue> {
        langfuse_attributes.to_vec()
    }

    fn enrich_attributes(&self, attributes: &[KeyValue]) -> Vec<KeyValue> {
        attributes.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bidirectional_mapping() {
        let mapper = GenAIAttributeMapper::new();

        // Test OTel to Langfuse
        let otel_attrs = vec![
            KeyValue::new(OpenTelemetryGenAIAttributes::REQUEST_MODEL, "gpt-4"),
            KeyValue::new("user.id", "user-123"),
        ];

        let langfuse_attrs = mapper.map_to_langfuse(&otel_attrs);

        assert!(langfuse_attrs
            .iter()
            .any(|kv| kv.key.as_str() == LangfuseAttributes::OBSERVATION_MODEL));
        assert!(langfuse_attrs
            .iter()
            .any(|kv| kv.key.as_str() == LangfuseAttributes::TRACE_USER_ID));

        // Test Langfuse to OTel (reverse)
        let otel_attrs_back = mapper.map_to_otel(&langfuse_attrs);

        assert!(otel_attrs_back
            .iter()
            .any(|kv| kv.key.as_str() == OpenTelemetryGenAIAttributes::REQUEST_MODEL));
    }

    #[test]
    fn test_token_usage_mapping() {
        let mapper = GenAIAttributeMapper::new();

        let otel_attrs = vec![
            KeyValue::new(OpenTelemetryGenAIAttributes::USAGE_PROMPT_TOKENS, 100i64),
            KeyValue::new(
                OpenTelemetryGenAIAttributes::USAGE_COMPLETION_TOKENS,
                200i64,
            ),
        ];

        let langfuse_attrs = mapper.map_to_langfuse(&otel_attrs);

        assert!(langfuse_attrs
            .iter()
            .any(|kv| kv.key.as_str() == LangfuseAttributes::OBSERVATION_USAGE_INPUT));
        assert!(langfuse_attrs
            .iter()
            .any(|kv| kv.key.as_str() == LangfuseAttributes::OBSERVATION_USAGE_OUTPUT));
    }

    #[test]
    fn test_model_parameters_aggregation() {
        let mapper = GenAIAttributeMapper::new();

        let otel_attrs = vec![
            KeyValue::new(OpenTelemetryGenAIAttributes::REQUEST_TEMPERATURE, 0.7),
            KeyValue::new(OpenTelemetryGenAIAttributes::REQUEST_MAX_TOKENS, 1000i64),
            KeyValue::new(OpenTelemetryGenAIAttributes::REQUEST_TOP_P, 0.9),
        ];

        let langfuse_attrs = mapper.map_to_langfuse(&otel_attrs);

        // Should have model parameters aggregated
        assert!(langfuse_attrs
            .iter()
            .any(|kv| kv.key.as_str() == LangfuseAttributes::OBSERVATION_MODEL_PARAMETERS));
    }

    #[test]
    fn test_enrich_attributes() {
        let mapper = GenAIAttributeMapper::new();

        let attrs = vec![
            KeyValue::new(OpenTelemetryGenAIAttributes::USAGE_PROMPT_TOKENS, 100i64),
            KeyValue::new(
                OpenTelemetryGenAIAttributes::USAGE_COMPLETION_TOKENS,
                200i64,
            ),
        ];

        let enriched = mapper.enrich_attributes(&attrs);

        // Should have added total tokens
        assert!(enriched
            .iter()
            .any(|kv| kv.key.as_str() == OpenTelemetryGenAIAttributes::USAGE_TOTAL_TOKENS));
        assert!(enriched
            .iter()
            .any(|kv| kv.key.as_str() == LangfuseAttributes::OBSERVATION_USAGE_TOTAL));

        // Check the value
        let total = enriched
            .iter()
            .find(|kv| kv.key.as_str() == OpenTelemetryGenAIAttributes::USAGE_TOTAL_TOKENS)
            .unwrap();

        if let opentelemetry::Value::I64(v) = total.value {
            assert_eq!(v, 300);
        } else {
            panic!("Expected I64 value");
        }
    }

    #[test]
    fn test_pass_through_mapper() {
        let mapper = PassThroughMapper;

        let attrs = vec![
            KeyValue::new("custom.attribute", "value"),
            KeyValue::new("another.attribute", 42i64),
        ];

        let mapped = mapper.map_to_langfuse(&attrs);
        assert_eq!(mapped, attrs);

        let mapped_back = mapper.map_to_otel(&mapped);
        assert_eq!(mapped_back, attrs);
    }
}
