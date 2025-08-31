//! Reproduction case for issue #10: NoHttpClient error when creating exporter
//! https://github.com/genai-rs/opentelemetry-langfuse/issues/10
//!
//! This example demonstrates that the bug has been fixed and exporters can now
//! be created successfully with the default HTTP client.

use opentelemetry_langfuse::ExporterBuilder;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("LANGFUSE_PUBLIC_KEY", "test-public-key");
    env::set_var("LANGFUSE_SECRET_KEY", "test-secret-key");
    env::set_var("LANGFUSE_HOST", "https://cloud.langfuse.com");

    println!("Attempting to create Langfuse exporter using ExporterBuilder...");

    let result = ExporterBuilder::new()
        .with_host("https://cloud.langfuse.com")
        .with_basic_auth("test-public-key", "test-secret-key")
        .build();

    match result {
        Ok(_) => println!("✅ Exporter created successfully!"),
        Err(e) => println!("❌ Failed to create exporter: {}", e),
    }

    println!("\nAttempting to create exporter from environment variables...");

    let result = opentelemetry_langfuse::exporter_from_langfuse_env();

    match result {
        Ok(_) => println!("✅ Exporter from env created successfully!"),
        Err(e) => println!("❌ Failed to create exporter from env: {}", e),
    }

    Ok(())
}
