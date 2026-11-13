# Trace attributes: `LangfuseContext`

Similar to the [langfuse-python SDK](https://langfuse.com/docs/sdk/python#update-trace),
this crate provides a `LangfuseContext` struct for managing trace-level attributes:

```rust
use opentelemetry_langfuse::LangfuseContext;

// Create a context instance
let context = LangfuseContext::new();

// Use fluent API for configuration
context
    .set_session_id("session-123")
    .set_user_id("user-456")
    .add_tags(vec!["production".to_string(), "api-v2".to_string()])
    .set_metadata(serde_json::json!({
        "environment": "production",
        "version": "1.0.0"
    }));

// Get attributes to add to spans
let attributes = context.get_attributes();
```

It also has `add_tag`, `set_trace_name`, `set_attribute`, `get_attribute`,
`has_attribute` and `clear`; see the
[API reference](https://docs.rs/opentelemetry-langfuse/latest/opentelemetry_langfuse/context/struct.LangfuseContext.html).

**Design note:** `LangfuseContext` uses an instance-based design (no global
state). Create instances as needed for your use case.

For an example integration, see the
[openai-ergonomic](https://github.com/genai-rs/openai-ergonomic) library's
`LangfuseInterceptor`. It illustrates attribute conventions; it does not imply
cross-version provider compatibility ([compatibility](compatibility.md)).
