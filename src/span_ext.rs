//! Extension traits for enriching OpenTelemetry spans with Langfuse attributes.

use opentelemetry::trace::Span;
use opentelemetry::KeyValue;
use serde_json::Value as JsonValue;

use crate::attributes::{LangfuseAttributes, OpenTelemetryGenAIAttributes};

/// Extension trait for adding Langfuse-specific attributes to OpenTelemetry spans.
pub trait LangfuseSpanExt {
    /// Adds Langfuse trace name
    fn set_trace_name(&mut self, name: impl Into<String>) -> &mut Self;

    /// Adds user ID
    fn set_user_id(&mut self, user_id: impl Into<String>) -> &mut Self;

    /// Adds session ID  
    fn set_session_id(&mut self, session_id: impl Into<String>) -> &mut Self;

    /// Adds trace tags
    fn set_trace_tags(&mut self, tags: Vec<String>) -> &mut Self;

    /// Adds trace metadata
    fn set_trace_metadata(&mut self, metadata: JsonValue) -> &mut Self;

    /// Adds observation model
    fn set_observation_model(&mut self, model: impl Into<String>) -> &mut Self;

    /// Adds observation type
    fn set_observation_type(&mut self, obs_type: impl Into<String>) -> &mut Self;

    /// Adds input usage tokens
    fn set_input_tokens(&mut self, tokens: i64) -> &mut Self;

    /// Adds output usage tokens
    fn set_output_tokens(&mut self, tokens: i64) -> &mut Self;

    /// Adds total usage tokens
    fn set_total_tokens(&mut self, tokens: i64) -> &mut Self;

    /// Adds model parameters as JSON
    fn set_model_parameters(&mut self, params: JsonValue) -> &mut Self;

    /// Adds arbitrary Langfuse metadata
    fn add_langfuse_metadata(&mut self, key: impl Into<String>, value: JsonValue) -> &mut Self;
}

impl<T: Span> LangfuseSpanExt for T {
    fn set_trace_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.set_attribute(KeyValue::new(LangfuseAttributes::TRACE_NAME, name.into()));
        self
    }

    fn set_user_id(&mut self, user_id: impl Into<String>) -> &mut Self {
        self.set_attribute(KeyValue::new(
            LangfuseAttributes::TRACE_USER_ID,
            user_id.into(),
        ));
        self
    }

    fn set_session_id(&mut self, session_id: impl Into<String>) -> &mut Self {
        self.set_attribute(KeyValue::new(
            LangfuseAttributes::TRACE_SESSION_ID,
            session_id.into(),
        ));
        self
    }

    fn set_trace_tags(&mut self, tags: Vec<String>) -> &mut Self {
        let tags_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
        self.set_attribute(KeyValue::new(LangfuseAttributes::TRACE_TAGS, tags_json));
        self
    }

    fn set_trace_metadata(&mut self, metadata: JsonValue) -> &mut Self {
        let metadata_str = metadata.to_string();
        self.set_attribute(KeyValue::new(
            LangfuseAttributes::TRACE_METADATA,
            metadata_str,
        ));
        self
    }

    fn set_observation_model(&mut self, model: impl Into<String>) -> &mut Self {
        let model_str = model.into();
        self.set_attribute(KeyValue::new(
            LangfuseAttributes::OBSERVATION_MODEL,
            model_str.clone(),
        ));
        self.set_attribute(KeyValue::new(
            OpenTelemetryGenAIAttributes::REQUEST_MODEL,
            model_str,
        ));
        self
    }

    fn set_observation_type(&mut self, obs_type: impl Into<String>) -> &mut Self {
        self.set_attribute(KeyValue::new(
            LangfuseAttributes::OBSERVATION_TYPE,
            obs_type.into(),
        ));
        self
    }

    fn set_input_tokens(&mut self, tokens: i64) -> &mut Self {
        self.set_attribute(KeyValue::new(
            LangfuseAttributes::OBSERVATION_USAGE_INPUT,
            tokens,
        ));
        self.set_attribute(KeyValue::new(
            OpenTelemetryGenAIAttributes::USAGE_PROMPT_TOKENS,
            tokens,
        ));
        self
    }

    fn set_output_tokens(&mut self, tokens: i64) -> &mut Self {
        self.set_attribute(KeyValue::new(
            LangfuseAttributes::OBSERVATION_USAGE_OUTPUT,
            tokens,
        ));
        self.set_attribute(KeyValue::new(
            OpenTelemetryGenAIAttributes::USAGE_COMPLETION_TOKENS,
            tokens,
        ));
        self
    }

    fn set_total_tokens(&mut self, tokens: i64) -> &mut Self {
        self.set_attribute(KeyValue::new(
            LangfuseAttributes::OBSERVATION_USAGE_TOTAL,
            tokens,
        ));
        self.set_attribute(KeyValue::new(
            OpenTelemetryGenAIAttributes::USAGE_TOTAL_TOKENS,
            tokens,
        ));
        self
    }

    fn set_model_parameters(&mut self, params: JsonValue) -> &mut Self {
        let params_str = params.to_string();
        self.set_attribute(KeyValue::new(
            LangfuseAttributes::OBSERVATION_MODEL_PARAMETERS,
            params_str,
        ));

        if let JsonValue::Object(map) = params {
            for (key, value) in map {
                let otel_key = format!("gen_ai.request.{}", key);
                match value {
                    JsonValue::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            self.set_attribute(KeyValue::new(otel_key, i));
                        } else if let Some(f) = n.as_f64() {
                            self.set_attribute(KeyValue::new(otel_key, f));
                        }
                    }
                    JsonValue::String(s) => {
                        self.set_attribute(KeyValue::new(otel_key, s));
                    }
                    JsonValue::Bool(b) => {
                        self.set_attribute(KeyValue::new(otel_key, b));
                    }
                    _ => {}
                }
            }
        }

        self
    }

    fn add_langfuse_metadata(&mut self, key: impl Into<String>, value: JsonValue) -> &mut Self {
        let full_key = format!("langfuse.metadata.{}", key.into());
        let value_str = value.to_string();
        self.set_attribute(KeyValue::new(full_key, value_str));
        self
    }
}

/// Extension trait for adding GenAI-specific attributes to OpenTelemetry spans.
pub trait GenAISpanExt {
    /// Sets the model name
    fn set_gen_ai_model(&mut self, model: impl Into<String>) -> &mut Self;

    /// Sets the temperature parameter
    fn set_gen_ai_temperature(&mut self, temperature: f64) -> &mut Self;

    /// Sets the max tokens parameter
    fn set_gen_ai_max_tokens(&mut self, max_tokens: i64) -> &mut Self;

    /// Sets the top_p parameter
    fn set_gen_ai_top_p(&mut self, top_p: f64) -> &mut Self;

    /// Sets the frequency penalty
    fn set_gen_ai_frequency_penalty(&mut self, penalty: f64) -> &mut Self;

    /// Sets the presence penalty
    fn set_gen_ai_presence_penalty(&mut self, penalty: f64) -> &mut Self;

    /// Sets prompt content with role
    fn set_gen_ai_prompt(
        &mut self,
        index: usize,
        role: impl Into<String>,
        content: impl Into<String>,
    ) -> &mut Self;

    /// Sets completion content with role
    fn set_gen_ai_completion(
        &mut self,
        index: usize,
        role: impl Into<String>,
        content: impl Into<String>,
    ) -> &mut Self;

    /// Sets token usage (prompt, completion, and calculates total)
    fn set_gen_ai_usage(&mut self, prompt_tokens: i64, completion_tokens: i64) -> &mut Self;
}

impl<T: Span> GenAISpanExt for T {
    fn set_gen_ai_model(&mut self, model: impl Into<String>) -> &mut Self {
        let model_str = model.into();
        self.set_attribute(KeyValue::new(
            OpenTelemetryGenAIAttributes::REQUEST_MODEL,
            model_str.clone(),
        ));
        self.set_attribute(KeyValue::new(
            LangfuseAttributes::OBSERVATION_MODEL,
            model_str,
        ));
        self
    }

    fn set_gen_ai_temperature(&mut self, temperature: f64) -> &mut Self {
        self.set_attribute(KeyValue::new(
            OpenTelemetryGenAIAttributes::REQUEST_TEMPERATURE,
            temperature,
        ));
        self
    }

    fn set_gen_ai_max_tokens(&mut self, max_tokens: i64) -> &mut Self {
        self.set_attribute(KeyValue::new(
            OpenTelemetryGenAIAttributes::REQUEST_MAX_TOKENS,
            max_tokens,
        ));
        self
    }

    fn set_gen_ai_top_p(&mut self, top_p: f64) -> &mut Self {
        self.set_attribute(KeyValue::new(
            OpenTelemetryGenAIAttributes::REQUEST_TOP_P,
            top_p,
        ));
        self
    }

    fn set_gen_ai_frequency_penalty(&mut self, penalty: f64) -> &mut Self {
        self.set_attribute(KeyValue::new(
            OpenTelemetryGenAIAttributes::REQUEST_FREQUENCY_PENALTY,
            penalty,
        ));
        self
    }

    fn set_gen_ai_presence_penalty(&mut self, penalty: f64) -> &mut Self {
        self.set_attribute(KeyValue::new(
            OpenTelemetryGenAIAttributes::REQUEST_PRESENCE_PENALTY,
            penalty,
        ));
        self
    }

    fn set_gen_ai_prompt(
        &mut self,
        index: usize,
        role: impl Into<String>,
        content: impl Into<String>,
    ) -> &mut Self {
        let role_key = format!("gen_ai.prompt.{}.role", index);
        let content_key = format!("gen_ai.prompt.{}.content", index);
        self.set_attribute(KeyValue::new(role_key, role.into()));
        self.set_attribute(KeyValue::new(content_key, content.into()));
        self
    }

    fn set_gen_ai_completion(
        &mut self,
        index: usize,
        role: impl Into<String>,
        content: impl Into<String>,
    ) -> &mut Self {
        let role_key = format!("gen_ai.completion.{}.role", index);
        let content_key = format!("gen_ai.completion.{}.content", index);
        self.set_attribute(KeyValue::new(role_key, role.into()));
        self.set_attribute(KeyValue::new(content_key, content.into()));
        self
    }

    fn set_gen_ai_usage(&mut self, prompt_tokens: i64, completion_tokens: i64) -> &mut Self {
        let total = prompt_tokens + completion_tokens;

        self.set_attribute(KeyValue::new(
            OpenTelemetryGenAIAttributes::USAGE_PROMPT_TOKENS,
            prompt_tokens,
        ));
        self.set_attribute(KeyValue::new(
            OpenTelemetryGenAIAttributes::USAGE_COMPLETION_TOKENS,
            completion_tokens,
        ));
        self.set_attribute(KeyValue::new(
            OpenTelemetryGenAIAttributes::USAGE_TOTAL_TOKENS,
            total,
        ));

        self.set_attribute(KeyValue::new(
            LangfuseAttributes::OBSERVATION_USAGE_INPUT,
            prompt_tokens,
        ));
        self.set_attribute(KeyValue::new(
            LangfuseAttributes::OBSERVATION_USAGE_OUTPUT,
            completion_tokens,
        ));
        self.set_attribute(KeyValue::new(
            LangfuseAttributes::OBSERVATION_USAGE_TOTAL,
            total,
        ));

        self
    }
}
