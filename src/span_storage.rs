//! Span storage for managing OpenTelemetry span lifecycle in interceptor patterns.
//!
//! This module provides a global registry for storing spans across async boundaries,
//! enabling proper span lifecycle management when span creation and completion
//! happen in separate calls (like in HTTP interceptors).
//!
//! # How It Works
//!
//! 1. In `before_request`: Create span, generate unique ID, store in registry
//! 2. Pass the span ID through request metadata
//! 3. In `after_response`: Retrieve span by ID, add final attributes, end span
//!
//! # Example
//!
//! ```no_run
//! use opentelemetry::trace::{Tracer, SpanKind};
//! use opentelemetry::global;
//! use opentelemetry::KeyValue;
//! use opentelemetry_langfuse::span_storage;
//!
//! # async fn example() {
//! let tracer = global::tracer("my-tracer");
//!
//! // In before_request interceptor:
//! let span_id = span_storage::create_and_store_span(
//!     &tracer,
//!     "my-operation",
//!     SpanKind::Client,
//!     vec![KeyValue::new("request.method", "POST")]
//! );
//!
//! // Store span_id in request metadata so after_response can find it
//! // metadata.insert("span_id", span_id);
//!
//! // ... make HTTP request ...
//!
//! // In after_response interceptor (retrieve span_id from metadata):
//! span_storage::add_span_attributes(&span_id, vec![
//!     KeyValue::new("response.status", "200")
//! ]);
//! span_storage::end_span(&span_id, vec![]);
//! # }
//! ```

use opentelemetry::trace::{SpanKind, TraceContextExt, Tracer};
use opentelemetry::{Context, KeyValue};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

lazy_static::lazy_static! {
    /// Global registry of active spans, keyed by unique span ID.
    static ref SPAN_REGISTRY: Arc<Mutex<HashMap<String, Context>>> = Arc::new(Mutex::new(HashMap::new()));
}

/// Generate a unique span ID.
fn generate_span_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("span-{}", timestamp)
}

/// Create a span and store it in the global registry.
///
/// Returns a unique span ID that can be used to retrieve and end the span later.
///
/// # Arguments
/// * `tracer` - The OpenTelemetry tracer to use
/// * `span_name` - Name for the span
/// * `kind` - The span kind (Client, Server, Internal, etc.)
/// * `attributes` - Initial attributes for the span
///
/// # Returns
/// A unique span ID string that must be passed to `add_span_attributes` and `end_span`
pub fn create_and_store_span<T>(
    tracer: &T,
    span_name: impl Into<String>,
    kind: SpanKind,
    attributes: Vec<KeyValue>,
) -> String
where
    T: Tracer,
    T::Span: Send + Sync + 'static,
{
    let span = tracer
        .span_builder(span_name.into())
        .with_kind(kind)
        .with_attributes(attributes)
        .start(tracer);

    let cx = Context::current_with_span(span);
    let span_id = generate_span_id();

    let mut registry = SPAN_REGISTRY.lock().unwrap();
    registry.insert(span_id.clone(), cx);

    span_id
}

/// Add attributes to a stored span.
///
/// # Arguments
/// * `span_id` - The span ID returned from `create_and_store_span`
/// * `attributes` - Attributes to add to the span
pub fn add_span_attributes(span_id: &str, attributes: Vec<KeyValue>) {
    let registry = SPAN_REGISTRY.lock().unwrap();
    if let Some(cx) = registry.get(span_id) {
        cx.span().set_attributes(attributes);
    }
}

/// Set the status of a stored span to error.
///
/// # Arguments
/// * `span_id` - The span ID returned from `create_and_store_span`
/// * `error_message` - Description of the error
pub fn set_span_error(span_id: &str, error_message: impl Into<String>) {
    use opentelemetry::trace::Status;
    let registry = SPAN_REGISTRY.lock().unwrap();
    if let Some(cx) = registry.get(span_id) {
        cx.span().set_status(Status::error(error_message.into()));
    }
}

/// End a stored span and remove it from the registry.
///
/// # Arguments
/// * `span_id` - The span ID returned from `create_and_store_span`
/// * `final_attributes` - Optional final attributes to add before ending
pub fn end_span(span_id: &str, final_attributes: Vec<KeyValue>) {
    let mut registry = SPAN_REGISTRY.lock().unwrap();
    if let Some(cx) = registry.remove(span_id) {
        let span = cx.span();
        if !final_attributes.is_empty() {
            span.set_attributes(final_attributes);
        }
        span.end();
    }
}

/// Check if a span exists in the registry.
pub fn has_span(span_id: &str) -> bool {
    let registry = SPAN_REGISTRY.lock().unwrap();
    registry.contains_key(span_id)
}

/// Get the current number of active spans in the registry.
///
/// Useful for debugging and monitoring.
pub fn active_span_count() -> usize {
    let registry = SPAN_REGISTRY.lock().unwrap();
    registry.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::trace::noop::NoopTracer;

    #[test]
    fn test_span_lifecycle() {
        let tracer = NoopTracer::new();

        // Create and store span
        let span_id = create_and_store_span(
            &tracer,
            "test-span",
            SpanKind::Client,
            vec![KeyValue::new("test", "value")],
        );

        // Verify span exists
        assert!(has_span(&span_id));
        assert_eq!(active_span_count(), 1);

        // Add attributes
        add_span_attributes(&span_id, vec![KeyValue::new("additional", "attr")]);

        // End span
        end_span(&span_id, vec![KeyValue::new("final", "attr")]);

        // Verify span is removed
        assert!(!has_span(&span_id));
        assert_eq!(active_span_count(), 0);
    }

    #[test]
    fn test_multiple_spans() {
        let tracer = NoopTracer::new();

        let span_id1 = create_and_store_span(&tracer, "span1", SpanKind::Client, vec![]);
        let span_id2 = create_and_store_span(&tracer, "span2", SpanKind::Client, vec![]);

        assert_eq!(active_span_count(), 2);
        assert!(has_span(&span_id1));
        assert!(has_span(&span_id2));

        end_span(&span_id1, vec![]);
        assert_eq!(active_span_count(), 1);
        assert!(!has_span(&span_id1));
        assert!(has_span(&span_id2));

        end_span(&span_id2, vec![]);
        assert_eq!(active_span_count(), 0);
    }

    #[test]
    fn test_nonexistent_span() {
        // These should not panic
        add_span_attributes("nonexistent", vec![]);
        end_span("nonexistent", vec![]);
        assert!(!has_span("nonexistent"));
    }
}
