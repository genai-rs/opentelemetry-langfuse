//! Task-local span storage for managing OpenTelemetry span lifecycle across async boundaries.
//!
//! This module provides utilities for storing and retrieving OpenTelemetry contexts
//! in task-local storage, enabling proper span lifecycle management in scenarios
//! where span creation and completion happen in separate async calls (like interceptors).
//!
//! # Example - Interceptor Pattern
//!
//! ```no_run
//! use opentelemetry::trace::{Tracer, TracerProvider, SpanKind};
//! use opentelemetry::global;
//! use opentelemetry::KeyValue;
//! use opentelemetry_langfuse::span_storage;
//!
//! # async fn example() {
//! let tracer = global::tracer("my-tracer");
//!
//! // Wrap your entire operation in with_storage
//! span_storage::with_storage(async {
//!     // In before_request interceptor:
//!     span_storage::create_and_store_span(
//!         &tracer,
//!         "my-operation",
//!         SpanKind::Client,
//!         vec![KeyValue::new("request.method", "POST")]
//!     );
//!
//!     // ... make HTTP request ...
//!
//!     // In after_response interceptor:
//!     span_storage::add_span_attributes(vec![
//!         KeyValue::new("response.status", "200")
//!     ]);
//!     span_storage::end_span_with_attributes(vec![]);
//! }).await;
//! # }
//! ```

use opentelemetry::trace::{SpanKind, TraceContextExt, Tracer};
use opentelemetry::{Context, KeyValue};
use std::cell::RefCell;

tokio::task_local! {
    /// Task-local storage for the current OpenTelemetry context.
    static CURRENT_CONTEXT: RefCell<Option<Context>>;
}

/// Execute a future with task-local span storage available.
///
/// This sets up the task-local storage scope that allows `store_span`,
/// `get_context`, and `take_span` to work within the future.
///
/// # Example
///
/// ```no_run
/// # use opentelemetry_langfuse::span_storage;
/// # async fn example() {
/// span_storage::with_storage(async {
///     // Your interceptor-based code here
///     // Can call store_span, get_context, take_span
/// }).await;
/// # }
/// ```
pub async fn with_storage<F, T>(f: F) -> T
where
    F: std::future::Future<Output = T>,
{
    CURRENT_CONTEXT.scope(RefCell::new(None), f).await
}

/// Store a span in task-local storage by wrapping it in a Context.
///
/// Must be called within a `with_storage` scope.
///
/// # Type Parameters
/// * `S` - The span type (must be Send + Sync + 'static)
pub fn store_span<S>(span: S)
where
    S: opentelemetry::trace::Span + Send + Sync + 'static,
{
    let cx = Context::current_with_span(span);
    if let Ok(cell) = CURRENT_CONTEXT.try_with(|c| {
        *c.borrow_mut() = Some(cx);
    }) {
        cell
    }
}

/// Get a reference to the current context from task-local storage.
///
/// Returns None if no context is stored or if called outside a `with_storage` scope.
pub fn get_context() -> Option<Context> {
    CURRENT_CONTEXT
        .try_with(|c| c.borrow().clone())
        .ok()
        .flatten()
}

/// Check if a span is currently stored.
///
/// Must be called within a `with_storage` scope.
pub fn has_span() -> bool {
    CURRENT_CONTEXT
        .try_with(|c| c.borrow().is_some())
        .unwrap_or(false)
}

/// Add attributes to the current span stored in task-local storage.
///
/// If no span is stored, this is a no-op.
/// Must be called within a `with_storage` scope.
pub fn add_span_attributes(attributes: Vec<KeyValue>) {
    let _ = CURRENT_CONTEXT.try_with(|c| {
        if let Some(cx) = c.borrow().as_ref() {
            cx.span().set_attributes(attributes);
        }
    });
}

/// Helper function to create and store a span in one call.
///
/// Must be called within a `with_storage` scope.
pub fn create_and_store_span<T>(
    tracer: &T,
    span_name: impl Into<String>,
    kind: SpanKind,
    attributes: Vec<KeyValue>,
) where
    T: Tracer,
    T::Span: Send + Sync + 'static,
{
    let span = tracer
        .span_builder(span_name.into())
        .with_kind(kind)
        .with_attributes(attributes)
        .start(tracer);

    store_span(span);
}

/// End the current span with optional final attributes.
///
/// Must be called within a `with_storage` scope.
pub fn end_span_with_attributes(attributes: Vec<KeyValue>) {
    let _ = CURRENT_CONTEXT.try_with(|c| {
        if let Some(cx) = c.borrow_mut().take() {
            let span = cx.span();
            if !attributes.is_empty() {
                span.set_attributes(attributes);
            }
            span.end();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::trace::noop::NoopTracer;

    #[tokio::test]
    async fn test_with_storage() {
        with_storage(async {
            let tracer = NoopTracer::new();
            let span = tracer.span_builder("test").start(&tracer);

            store_span(span);

            // Verify we can retrieve the context
            let ctx = get_context();
            assert!(ctx.is_some());
        })
        .await;
    }

    #[tokio::test]
    async fn test_create_and_store() {
        with_storage(async {
            let tracer = NoopTracer::new();

            create_and_store_span(&tracer, "test-span", SpanKind::Client, vec![]);

            let ctx = get_context();
            assert!(ctx.is_some());
        })
        .await;
    }

    #[tokio::test]
    async fn test_add_attributes() {
        with_storage(async {
            let tracer = NoopTracer::new();
            let span = tracer.span_builder("test").start(&tracer);

            store_span(span);
            add_span_attributes(vec![KeyValue::new("test", "value")]);

            // No assertion needed - just verify it doesn't panic
        })
        .await;
    }

    #[tokio::test]
    async fn test_end_span() {
        with_storage(async {
            let tracer = NoopTracer::new();
            let span = tracer.span_builder("test").start(&tracer);

            store_span(span);
            end_span_with_attributes(vec![KeyValue::new("final", "attr")]);

            // Verify span was taken
            let ctx = get_context();
            assert!(ctx.is_none());
        })
        .await;
    }

    #[tokio::test]
    async fn test_outside_scope() {
        // These should not panic when called outside with_storage
        let ctx = get_context();
        assert!(ctx.is_none());

        add_span_attributes(vec![]);
        end_span_with_attributes(vec![]);
    }
}
