//! OpenTelemetry integration for Langfuse.
//!
//! This crate provides OpenTelemetry components and utilities for integrating
//! with Langfuse, enabling comprehensive observability for LLM applications.
//!
//! # Features
//!
//! - **Attribute Mapping**: Bidirectional mapping between Langfuse and OpenTelemetry GenAI conventions
//! - **Context Management**: Thread-safe, explicit context passing without global state
//! - **Custom Span Processing**: Enriches spans with Langfuse-specific attributes
//! - **Builder Pattern**: Fluent API for configuring tracers and contexts
//! - **GenAI Support**: Full support for OpenTelemetry GenAI semantic conventions
//!
//! # Quick Start
//!
//! ```no_run
//! use opentelemetry_langfuse::{LangfuseSpanExt, GenAISpanExt};
//! use opentelemetry::trace::{Tracer, Span};
//!
//! # fn example<T: Tracer>(tracer: &T) {
//! // Create a span using your existing OpenTelemetry tracer
//! let mut span = tracer.span_builder("chat.completion").start(tracer);
//!
//! // Enrich with Langfuse attributes using extension methods
//! span.set_user_id("user-456")
//!     .set_session_id("session-123")
//!     .set_observation_model("gpt-4")
//!     .set_observation_type("generation");
//!
//! // Add GenAI semantic convention attributes
//! span.set_gen_ai_temperature(0.7)
//!     .set_gen_ai_max_tokens(1000)
//!     .set_gen_ai_usage(150, 250); // prompt and completion tokens
//!
//! // ... your LLM call here ...
//! span.end();
//! # }
//! ```
//!
//! # Span Enrichment
//!
//! The library provides extension traits to easily add Langfuse and GenAI attributes to any OpenTelemetry span:
//!
//! ```no_run
//! use opentelemetry_langfuse::{LangfuseSpanExt, GenAISpanExt};
//! use opentelemetry::trace::{Tracer, Span};
//! use serde_json::json;
//!
//! # fn example<T: Tracer>(tracer: &T) {
//! let mut span = tracer.span_builder("llm.request").start(tracer);
//!
//! // Add Langfuse-specific attributes
//! span.set_trace_name("Customer Support Chat")
//!     .set_trace_tags(vec!["support".to_string(), "chat".to_string()])
//!     .add_langfuse_metadata("priority", json!("high"));
//!
//! // Add GenAI semantic convention attributes  
//! span.set_gen_ai_prompt(0, "system", "You are a helpful assistant")
//!     .set_gen_ai_prompt(1, "user", "What's the weather?")
//!     .set_gen_ai_completion(0, "assistant", "I can help with that...");
//! # }
//! ```
//!
//! # Attribute Mapping
//!
//! Automatic mapping between Langfuse and OpenTelemetry conventions:
//!
//! ```no_run
//! use opentelemetry_langfuse::mapper::{GenAIAttributeMapper, AttributeMapper};
//! use opentelemetry::KeyValue;
//!
//! let mapper = GenAIAttributeMapper::new();
//!
//! // OpenTelemetry attributes
//! let otel_attrs = vec![
//!     KeyValue::new("gen_ai.request.model", "gpt-4"),
//!     KeyValue::new("gen_ai.usage.prompt_tokens", 150i64),
//! ];
//!
//! // Automatically mapped to Langfuse format
//! let langfuse_attrs = mapper.map_to_langfuse(&otel_attrs);
//! ```

pub mod attributes;
pub mod context;
pub mod mapper;
pub mod processor;
pub mod span_ext;

pub mod auth;
pub mod constants;
pub mod endpoint;
pub mod error;
pub mod exporter;

pub use attributes::{
    LangfuseAttributes, ObservationAttributesBuilder, OpenTelemetryGenAIAttributes,
    TraceAttributesBuilder,
};
pub use context::{TracingContext, TracingContextBuilder};
pub use mapper::{AttributeMapper, GenAIAttributeMapper, MappingRule, PassThroughMapper};
pub use processor::MappingExporter;
pub use span_ext::{GenAISpanExt, LangfuseSpanExt};

pub use auth::{build_auth_header, build_auth_header_from_env};
pub use endpoint::{build_otlp_endpoint, build_otlp_endpoint_from_env};
pub use error::{Error, Result};
pub use exporter::{
    exporter, exporter_from_env, exporter_from_langfuse_env, exporter_from_otel_env,
    ExporterBuilder,
};
