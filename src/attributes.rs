//! Attribute definitions for Langfuse and OpenTelemetry GenAI semantic conventions.
//!
//! This module provides constant definitions for both Langfuse-specific attributes
//! and OpenTelemetry GenAI semantic conventions, along with builder patterns for
//! constructing attribute sets.
//!
//! When the `gen-ai` feature is enabled in opentelemetry-semantic-conventions,
//! we use the official constants. Otherwise, we define our own for compatibility.

use opentelemetry::KeyValue;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

// Note: Since this is an OpenTelemetry-Langfuse extension specifically for GenAI,
// we assume GenAI constants will be available in future versions of the semantic conventions crate.
// For now, we define our own constants to maintain compatibility.

/// Langfuse-specific attribute names.
///
/// These attributes are used to map OpenTelemetry span attributes directly
/// to the Langfuse data model. They always take precedence over generic
/// OpenTelemetry conventions.
pub struct LangfuseAttributes;

impl LangfuseAttributes {
    // Trace attributes
    /// Name of the trace
    pub const TRACE_NAME: &'static str = "langfuse.trace.name";

    /// Unique identifier for the end-user
    pub const TRACE_USER_ID: &'static str = "user.id";

    /// Unique identifier for the user session
    pub const TRACE_SESSION_ID: &'static str = "session.id";

    /// Tags associated with the trace
    pub const TRACE_TAGS: &'static str = "langfuse.trace.tags";

    /// Whether the trace is public
    pub const TRACE_PUBLIC: &'static str = "langfuse.trace.public";

    /// Metadata for the trace
    pub const TRACE_METADATA: &'static str = "langfuse.trace.metadata";

    /// Input data for the trace
    pub const TRACE_INPUT: &'static str = "langfuse.trace.input";

    /// Output data for the trace
    pub const TRACE_OUTPUT: &'static str = "langfuse.trace.output";

    // Observation attributes
    /// Type of observation (e.g., "generation", "span", "event")
    pub const OBSERVATION_TYPE: &'static str = "langfuse.observation.type";

    /// Metadata for the observation
    pub const OBSERVATION_METADATA: &'static str = "langfuse.observation.metadata";

    /// Name of the generative model used
    pub const OBSERVATION_MODEL: &'static str = "langfuse.observation.model.name";

    /// Model parameters as JSON string
    pub const OBSERVATION_MODEL_PARAMETERS: &'static str = "langfuse.observation.model.parameters";

    /// Input data for the observation
    pub const OBSERVATION_INPUT: &'static str = "langfuse.observation.input";

    /// Output data for the observation
    pub const OBSERVATION_OUTPUT: &'static str = "langfuse.observation.output";

    /// Completion start time
    pub const OBSERVATION_COMPLETION_START_TIME: &'static str =
        "langfuse.observation.completion_start_time";

    /// Token usage information
    pub const OBSERVATION_USAGE_INPUT: &'static str = "langfuse.observation.usage.input";
    pub const OBSERVATION_USAGE_OUTPUT: &'static str = "langfuse.observation.usage.output";
    pub const OBSERVATION_USAGE_TOTAL: &'static str = "langfuse.observation.usage.total";
}

/// OpenTelemetry GenAI semantic convention attributes.
///
/// These follow the OpenTelemetry specification for generative AI systems.
/// When the official constants are available, we use them. Otherwise, we provide
/// our own definitions for compatibility.
/// See: https://opentelemetry.io/docs/specs/semconv/gen-ai/
pub struct OpenTelemetryGenAIAttributes;

impl OpenTelemetryGenAIAttributes {
    // GenAI semantic convention constants
    // These follow the OpenTelemetry specification for generative AI systems.
    // Once the official constants are available in the semantic conventions crate,
    // we can switch to using those.
    
    // Request attributes
    /// The name of the GenAI model being used
    pub const REQUEST_MODEL: &'static str = "gen_ai.request.model";
    
    /// Temperature setting for the model
    pub const REQUEST_TEMPERATURE: &'static str = "gen_ai.request.temperature";
    
    /// Maximum tokens to generate
    pub const REQUEST_MAX_TOKENS: &'static str = "gen_ai.request.max_tokens";
    
    /// Top-p sampling parameter
    pub const REQUEST_TOP_P: &'static str = "gen_ai.request.top_p";
    
    /// Top-k sampling parameter
    pub const REQUEST_TOP_K: &'static str = "gen_ai.request.top_k";
    
    /// Stop sequences
    pub const REQUEST_STOP_SEQUENCES: &'static str = "gen_ai.request.stop_sequences";
    
    /// Frequency penalty
    pub const REQUEST_FREQUENCY_PENALTY: &'static str = "gen_ai.request.frequency_penalty";
    
    /// Presence penalty
    pub const REQUEST_PRESENCE_PENALTY: &'static str = "gen_ai.request.presence_penalty";
    
    // Response attributes
    /// The unique identifier of the response
    pub const RESPONSE_ID: &'static str = "gen_ai.response.id";
    
    /// The model used for the response
    pub const RESPONSE_MODEL: &'static str = "gen_ai.response.model";
    
    /// Finish reasons for the response
    pub const RESPONSE_FINISH_REASONS: &'static str = "gen_ai.response.finish_reasons";
    
    // Usage attributes
    /// Number of tokens in the prompt (input)
    /// Note: The spec is moving towards gen_ai.usage.input_tokens
    pub const USAGE_PROMPT_TOKENS: &'static str = "gen_ai.usage.prompt_tokens";
    
    /// Number of tokens in the completion (output)
    /// Note: The spec is moving towards gen_ai.usage.output_tokens
    pub const USAGE_COMPLETION_TOKENS: &'static str = "gen_ai.usage.completion_tokens";
    
    /// Total number of tokens used
    pub const USAGE_TOTAL_TOKENS: &'static str = "gen_ai.usage.total_tokens";
    
    // Prompt and completion attributes
    /// JSON-serialized prompts
    pub const PROMPT_JSON: &'static str = "gen_ai.prompt_json";
    
    /// JSON-serialized completions
    pub const COMPLETION_JSON: &'static str = "gen_ai.completion_json";
    
    // Individual prompt/completion attributes (0-indexed)
    /// Role of the first prompt
    pub const PROMPT_0_ROLE: &'static str = "gen_ai.prompt.0.role";
    
    /// Content of the first prompt
    pub const PROMPT_0_CONTENT: &'static str = "gen_ai.prompt.0.content";
    
    /// Role of the first completion
    pub const COMPLETION_0_ROLE: &'static str = "gen_ai.completion.0.role";
    
    /// Content of the first completion
    pub const COMPLETION_0_CONTENT: &'static str = "gen_ai.completion.0.content";
    
    // System attributes
    /// The GenAI system being used (e.g., "openai", "anthropic")
    pub const SYSTEM: &'static str = "gen_ai.system";
    
    /// The type of GenAI operation
    pub const OPERATION_NAME: &'static str = "gen_ai.operation.name";
}

/// Builder for constructing trace attributes.
#[derive(Default, Clone)]
pub struct TraceAttributesBuilder {
    attributes: HashMap<String, JsonValue>,
}

impl TraceAttributesBuilder {
    /// Creates a new trace attributes builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the trace name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.attributes.insert(
            LangfuseAttributes::TRACE_NAME.to_string(),
            JsonValue::String(name.into()),
        );
        self
    }

    /// Sets the user ID.
    pub fn user_id(mut self, user_id: impl Into<String>) -> Self {
        self.attributes.insert(
            LangfuseAttributes::TRACE_USER_ID.to_string(),
            JsonValue::String(user_id.into()),
        );
        self
    }

    /// Sets the session ID.
    pub fn session_id(mut self, session_id: impl Into<String>) -> Self {
        self.attributes.insert(
            LangfuseAttributes::TRACE_SESSION_ID.to_string(),
            JsonValue::String(session_id.into()),
        );
        self
    }

    /// Adds tags to the trace.
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.attributes.insert(
            LangfuseAttributes::TRACE_TAGS.to_string(),
            JsonValue::Array(tags.into_iter().map(JsonValue::String).collect()),
        );
        self
    }

    /// Sets whether the trace is public.
    pub fn public(mut self, is_public: bool) -> Self {
        self.attributes.insert(
            LangfuseAttributes::TRACE_PUBLIC.to_string(),
            JsonValue::Bool(is_public),
        );
        self
    }

    /// Adds metadata to the trace.
    pub fn metadata(mut self, key: impl Into<String>, value: JsonValue) -> Self {
        let metadata_key = format!("{}.{}", LangfuseAttributes::TRACE_METADATA, key.into());
        self.attributes.insert(metadata_key, value);
        self
    }

    /// Sets the input data.
    pub fn input(mut self, input: JsonValue) -> Self {
        self.attributes
            .insert(LangfuseAttributes::TRACE_INPUT.to_string(), input);
        self
    }

    /// Sets the output data.
    pub fn output(mut self, output: JsonValue) -> Self {
        self.attributes
            .insert(LangfuseAttributes::TRACE_OUTPUT.to_string(), output);
        self
    }

    /// Builds the attributes as OpenTelemetry KeyValue pairs.
    pub fn build(self) -> Vec<KeyValue> {
        self.attributes
            .into_iter()
            .map(|(k, v)| match v {
                JsonValue::String(s) => KeyValue::new(k, s),
                JsonValue::Bool(b) => KeyValue::new(k, b),
                JsonValue::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        KeyValue::new(k, i)
                    } else if let Some(f) = n.as_f64() {
                        KeyValue::new(k, f)
                    } else {
                        KeyValue::new(k, n.to_string())
                    }
                }
                _ => KeyValue::new(k, v.to_string()),
            })
            .collect()
    }
}

/// Builder for constructing observation attributes.
#[derive(Default, Clone)]
pub struct ObservationAttributesBuilder {
    attributes: HashMap<String, JsonValue>,
}

impl ObservationAttributesBuilder {
    /// Creates a new observation attributes builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the observation type.
    pub fn observation_type(mut self, obs_type: impl Into<String>) -> Self {
        self.attributes.insert(
            LangfuseAttributes::OBSERVATION_TYPE.to_string(),
            JsonValue::String(obs_type.into()),
        );
        self
    }

    /// Sets the model name.
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.attributes.insert(
            LangfuseAttributes::OBSERVATION_MODEL.to_string(),
            JsonValue::String(model.into()),
        );
        self
    }

    /// Sets the model parameters.
    pub fn model_parameters(mut self, params: JsonValue) -> Self {
        self.attributes.insert(
            LangfuseAttributes::OBSERVATION_MODEL_PARAMETERS.to_string(),
            params,
        );
        self
    }

    /// Sets the input data.
    pub fn input(mut self, input: JsonValue) -> Self {
        self.attributes
            .insert(LangfuseAttributes::OBSERVATION_INPUT.to_string(), input);
        self
    }

    /// Sets the output data.
    pub fn output(mut self, output: JsonValue) -> Self {
        self.attributes
            .insert(LangfuseAttributes::OBSERVATION_OUTPUT.to_string(), output);
        self
    }

    /// Sets token usage information.
    pub fn usage(mut self, input_tokens: i64, output_tokens: i64) -> Self {
        self.attributes.insert(
            LangfuseAttributes::OBSERVATION_USAGE_INPUT.to_string(),
            JsonValue::Number(input_tokens.into()),
        );
        self.attributes.insert(
            LangfuseAttributes::OBSERVATION_USAGE_OUTPUT.to_string(),
            JsonValue::Number(output_tokens.into()),
        );
        self.attributes.insert(
            LangfuseAttributes::OBSERVATION_USAGE_TOTAL.to_string(),
            JsonValue::Number((input_tokens + output_tokens).into()),
        );
        self
    }

    /// Adds metadata to the observation.
    pub fn metadata(mut self, key: impl Into<String>, value: JsonValue) -> Self {
        let metadata_key = format!(
            "{}.{}",
            LangfuseAttributes::OBSERVATION_METADATA,
            key.into()
        );
        self.attributes.insert(metadata_key, value);
        self
    }

    /// Builds the attributes as OpenTelemetry KeyValue pairs.
    pub fn build(self) -> Vec<KeyValue> {
        self.attributes
            .into_iter()
            .map(|(k, v)| match v {
                JsonValue::String(s) => KeyValue::new(k, s),
                JsonValue::Bool(b) => KeyValue::new(k, b),
                JsonValue::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        KeyValue::new(k, i)
                    } else if let Some(f) = n.as_f64() {
                        KeyValue::new(k, f)
                    } else {
                        KeyValue::new(k, n.to_string())
                    }
                }
                _ => KeyValue::new(k, v.to_string()),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_attributes_builder() {
        let attributes = TraceAttributesBuilder::new()
            .name("test-trace")
            .user_id("user-123")
            .session_id("session-456")
            .tags(vec!["tag1".to_string(), "tag2".to_string()])
            .public(true)
            .build();

        assert_eq!(attributes.len(), 5);
        assert!(attributes
            .iter()
            .any(|kv| kv.key.as_str() == LangfuseAttributes::TRACE_NAME));
        assert!(attributes
            .iter()
            .any(|kv| kv.key.as_str() == LangfuseAttributes::TRACE_USER_ID));
    }

    #[test]
    fn test_observation_attributes_builder() {
        let attributes = ObservationAttributesBuilder::new()
            .observation_type("generation")
            .model("gpt-4")
            .usage(100, 200)
            .build();

        assert_eq!(attributes.len(), 5); // type, model, input tokens, output tokens, total tokens
        assert!(attributes
            .iter()
            .any(|kv| kv.key.as_str() == LangfuseAttributes::OBSERVATION_TYPE));
        assert!(attributes
            .iter()
            .any(|kv| kv.key.as_str() == LangfuseAttributes::OBSERVATION_MODEL));
    }
}
