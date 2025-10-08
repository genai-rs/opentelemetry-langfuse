//! Langfuse context helpers for setting trace attributes.
//!
//! This module provides a `LangfuseContext` struct that can be used to store
//! trace-level attributes. This is typically used with interceptors or middleware
//! where each instance maintains its own context.
//!
//! # Example
//!
//! ```no_run
//! use opentelemetry_langfuse::LangfuseContext;
//!
//! // Create a context instance
//! let context = LangfuseContext::new();
//!
//! // Set session ID for grouping traces
//! context.set_session_id("session-123");
//!
//! // Set user ID for attribution
//! context.set_user_id("user-456");
//!
//! // Add tags for filtering
//! context.add_tags(vec!["production".to_string(), "api-v2".to_string()]);
//!
//! // Get attributes to add to spans
//! let attributes = context.get_attributes();
//! ```

use opentelemetry::KeyValue;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Langfuse-specific attribute keys.
///
/// These follow the Langfuse OpenTelemetry conventions documented at:
/// <https://langfuse.com/integrations/native/opentelemetry>
pub mod attributes {
    /// Session ID attribute key (Langfuse expects `langfuse.session.id` or `session.id`)
    pub const TRACE_SESSION_ID: &str = "langfuse.session.id";
    /// User ID attribute key (Langfuse expects `langfuse.user.id` or `user.id`)
    pub const TRACE_USER_ID: &str = "langfuse.user.id";
    /// Tags attribute key - must be a string array (Langfuse expects `langfuse.trace.tags`)
    pub const TRACE_TAGS: &str = "langfuse.trace.tags";
    /// Metadata attribute key (JSON object string)
    pub const TRACE_METADATA: &str = "langfuse.trace.metadata";
    /// Trace name attribute key
    pub const TRACE_NAME: &str = "langfuse.trace.name";
}

/// Thread-safe storage for Langfuse context attributes.
///
/// This context allows you to set attributes that will be automatically
/// included in all spans created within the same context.
#[derive(Clone)]
pub struct LangfuseContext {
    attributes: Arc<RwLock<HashMap<String, String>>>,
}

impl LangfuseContext {
    /// Create a new empty context.
    #[must_use]
    pub fn new() -> Self {
        Self {
            attributes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set the session ID for the current trace.
    pub fn set_session_id(&self, session_id: impl Into<String>) -> &Self {
        self.set_attribute(attributes::TRACE_SESSION_ID, session_id);
        self
    }

    /// Set the user ID for the current trace.
    pub fn set_user_id(&self, user_id: impl Into<String>) -> &Self {
        self.set_attribute(attributes::TRACE_USER_ID, user_id);
        self
    }

    /// Add tags to the current trace.
    ///
    /// Tags are stored as a JSON array string.
    pub fn add_tags(&self, tags: Vec<String>) -> &Self {
        let tags_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
        self.set_attribute(attributes::TRACE_TAGS, tags_json);
        self
    }

    /// Add a single tag.
    pub fn add_tag(&self, tag: impl Into<String>) -> &Self {
        let tag = tag.into();
        let mut attrs = self.attributes.write().unwrap();

        // Append to existing tags if present
        if let Some(existing) = attrs.get(attributes::TRACE_TAGS) {
            // Parse existing JSON array, add tag, and re-serialize
            if let Ok(mut tags_vec) = serde_json::from_str::<Vec<String>>(existing) {
                tags_vec.push(tag);
                let tags_json =
                    serde_json::to_string(&tags_vec).unwrap_or_else(|_| "[]".to_string());
                attrs.insert(attributes::TRACE_TAGS.to_string(), tags_json);
            } else {
                // Fallback if existing isn't valid JSON
                attrs.insert(attributes::TRACE_TAGS.to_string(), format!("[\"{}\"]", tag));
            }
        } else {
            attrs.insert(attributes::TRACE_TAGS.to_string(), format!("[\"{}\"]", tag));
        }
        drop(attrs);
        self
    }

    /// Set metadata as JSON string.
    pub fn set_metadata(&self, metadata: serde_json::Value) -> &Self {
        let metadata_str = metadata.to_string();
        self.set_attribute(attributes::TRACE_METADATA, metadata_str);
        self
    }

    /// Set a custom attribute.
    pub fn set_attribute(&self, key: impl Into<String>, value: impl Into<String>) -> &Self {
        let mut attrs = self.attributes.write().unwrap();
        attrs.insert(key.into(), value.into());
        self
    }

    /// Set the trace name.
    pub fn set_trace_name(&self, name: impl Into<String>) -> &Self {
        self.set_attribute(attributes::TRACE_NAME, name);
        self
    }

    /// Clear all attributes.
    pub fn clear(&self) {
        let mut attrs = self.attributes.write().unwrap();
        attrs.clear();
    }

    /// Get all current attributes as key-value pairs.
    #[must_use]
    pub fn get_attributes(&self) -> Vec<KeyValue> {
        let attrs = self.attributes.read().unwrap();
        attrs
            .iter()
            .map(|(k, v)| KeyValue::new(k.clone(), v.clone()))
            .collect()
    }

    /// Check if a specific attribute is set.
    #[must_use]
    pub fn has_attribute(&self, key: &str) -> bool {
        let attrs = self.attributes.read().unwrap();
        attrs.contains_key(key)
    }

    /// Get a specific attribute value.
    #[must_use]
    pub fn get_attribute(&self, key: &str) -> Option<String> {
        let attrs = self.attributes.read().unwrap();
        attrs.get(key).cloned()
    }
}

impl Default for LangfuseContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder pattern for fluent API.
pub struct LangfuseContextBuilder {
    context: LangfuseContext,
}

impl Default for LangfuseContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl LangfuseContextBuilder {
    /// Create a new builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            context: LangfuseContext::new(),
        }
    }

    /// Set session ID.
    #[must_use]
    pub fn session_id(self, session_id: impl Into<String>) -> Self {
        self.context.set_session_id(session_id);
        self
    }

    /// Set user ID.
    #[must_use]
    pub fn user_id(self, user_id: impl Into<String>) -> Self {
        self.context.set_user_id(user_id);
        self
    }

    /// Add tags.
    #[must_use]
    pub fn tags(self, tags: Vec<String>) -> Self {
        self.context.add_tags(tags);
        self
    }

    /// Set metadata.
    #[must_use]
    pub fn metadata(self, metadata: serde_json::Value) -> Self {
        self.context.set_metadata(metadata);
        self
    }

    /// Set trace name.
    #[must_use]
    pub fn trace_name(self, name: impl Into<String>) -> Self {
        self.context.set_trace_name(name);
        self
    }

    /// Build the context.
    #[must_use]
    pub fn build(self) -> LangfuseContext {
        self.context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_attributes() {
        let ctx = LangfuseContext::new();
        ctx.set_session_id("session-123");
        ctx.set_user_id("user-456");

        assert_eq!(
            ctx.get_attribute(attributes::TRACE_SESSION_ID),
            Some("session-123".to_string())
        );
        assert_eq!(
            ctx.get_attribute(attributes::TRACE_USER_ID),
            Some("user-456".to_string())
        );
    }

    #[test]
    fn test_tags() {
        let ctx = LangfuseContext::new();
        ctx.add_tags(vec!["tag1".to_string(), "tag2".to_string()]);

        let tags_json = ctx.get_attribute(attributes::TRACE_TAGS).unwrap();
        let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap();
        assert_eq!(tags, vec!["tag1", "tag2"]);
    }

    #[test]
    fn test_add_single_tag() {
        let ctx = LangfuseContext::new();
        ctx.add_tag("tag1");
        ctx.add_tag("tag2");

        let tags_json = ctx.get_attribute(attributes::TRACE_TAGS).unwrap();
        let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap();
        assert_eq!(tags, vec!["tag1", "tag2"]);
    }

    #[test]
    fn test_builder() {
        let ctx = LangfuseContextBuilder::new()
            .session_id("session-123")
            .user_id("user-456")
            .tags(vec!["tag1".to_string()])
            .trace_name("my-trace")
            .build();

        assert!(ctx.has_attribute(attributes::TRACE_SESSION_ID));
        assert!(ctx.has_attribute(attributes::TRACE_USER_ID));
        assert!(ctx.has_attribute(attributes::TRACE_TAGS));
        assert!(ctx.has_attribute(attributes::TRACE_NAME));
    }

    #[test]
    fn test_clear() {
        let ctx = LangfuseContext::new();
        ctx.set_session_id("session-123");
        assert!(ctx.has_attribute(attributes::TRACE_SESSION_ID));

        ctx.clear();
        assert!(!ctx.has_attribute(attributes::TRACE_SESSION_ID));
    }
}
