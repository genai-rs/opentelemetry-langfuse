//! Example using standard OpenTelemetry environment variables.

use base64::Engine;
use langfuse_ergonomic::client::LangfuseClient;
use opentelemetry::global;
use opentelemetry::trace::{Span, Tracer};
use opentelemetry::KeyValue;
use opentelemetry_langfuse::exporter_from_otel_env;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing for debugging
    tracing_subscriber::fmt::init();

    println!("Testing OpenTelemetry environment variables with Langfuse...\n");

    // Test both OTEL endpoint configurations
    test_traces_endpoint().await?;
    println!("\n---\n");
    test_base_endpoint().await?;

    Ok(())
}

async fn test_traces_endpoint() -> Result<(), Box<dyn Error>> {
    println!("Test 1: Using OTEL_EXPORTER_OTLP_TRACES_ENDPOINT");
    
    // Get Langfuse credentials from environment
    let public_key = std::env::var("LANGFUSE_PUBLIC_KEY")
        .expect("LANGFUSE_PUBLIC_KEY must be set");
    let secret_key = std::env::var("LANGFUSE_SECRET_KEY")
        .expect("LANGFUSE_SECRET_KEY must be set");
    let host = std::env::var("LANGFUSE_HOST")
        .unwrap_or_else(|_| "https://cloud.langfuse.com".to_string());
    
    // Create base64 encoded credentials for OTEL
    let credentials = base64::engine::general_purpose::STANDARD
        .encode(format!("{}:{}", public_key, secret_key));
    
    // Set OTEL environment variables with traces endpoint
    std::env::set_var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT", format!("{}/api/public/otel", host));
    std::env::set_var("OTEL_EXPORTER_OTLP_TRACES_HEADERS", format!("Authorization=Basic {}", credentials));
    
    println!("  OTEL_EXPORTER_OTLP_TRACES_ENDPOINT={}/api/public/otel", host);
    println!("  OTEL_EXPORTER_OTLP_TRACES_HEADERS=Authorization=Basic <credentials>");
    
    // Create exporter using OTEL environment variables
    let exporter = exporter_from_otel_env()?;
    
    // Create tracer provider
    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(Resource::builder().with_attributes(vec![
            KeyValue::new("service.name", "otel-env-test-traces-endpoint"),
            KeyValue::new("test.type", "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT"),
        ]).build())
        .build();
    
    global::set_tracer_provider(provider.clone());
    
    // Create test trace
    let tracer = global::tracer("otel-test-traces");
    let mut span = tracer
        .span_builder("test-otel-traces-endpoint")
        .with_attributes(vec![
            KeyValue::new("test.endpoint_type", "traces_endpoint"),
            KeyValue::new("test.config", "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT"),
        ])
        .start(&tracer);
    
    span.add_event("Testing OTEL_EXPORTER_OTLP_TRACES_ENDPOINT configuration", vec![]);
    span.end();
    
    println!("  Trace created: test-otel-traces-endpoint");
    
    // Give time for export
    sleep(Duration::from_secs(2)).await;
    drop(provider);
    sleep(Duration::from_secs(1)).await;
    
    // Verify in Langfuse
    println!("  Verifying trace in Langfuse...");
    // Set Langfuse env vars temporarily for the client
    std::env::set_var("LANGFUSE_HOST", &host);
    std::env::set_var("LANGFUSE_PUBLIC_KEY", &public_key);
    std::env::set_var("LANGFUSE_SECRET_KEY", &secret_key);
    let client = LangfuseClient::from_env()?;
    std::env::remove_var("LANGFUSE_HOST");
    std::env::remove_var("LANGFUSE_PUBLIC_KEY");
    std::env::remove_var("LANGFUSE_SECRET_KEY");
    
    let response = client.list_traces().limit(5).call().await?;
    let found = if let Some(data) = response.get("data") {
        if let Some(array) = data.as_array() {
            array.iter().any(|t| 
                t.get("name")
                    .and_then(|n| n.as_str())
                    .map(|n| n.contains("test-otel-traces-endpoint"))
                    .unwrap_or(false)
            )
        } else {
            false
        }
    } else {
        false
    };
    
    if found {
        println!("  ✅ Trace found in Langfuse!");
    } else {
        println!("  ⚠️  Trace not found yet (may need more time to process)");
    }
    
    // Clean up
    std::env::remove_var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT");
    std::env::remove_var("OTEL_EXPORTER_OTLP_TRACES_HEADERS");
    
    Ok(())
}

async fn test_base_endpoint() -> Result<(), Box<dyn Error>> {
    println!("Test 2: Using OTEL_EXPORTER_OTLP_ENDPOINT");
    
    // Get Langfuse credentials from environment
    let public_key = std::env::var("LANGFUSE_PUBLIC_KEY")
        .expect("LANGFUSE_PUBLIC_KEY must be set");
    let secret_key = std::env::var("LANGFUSE_SECRET_KEY")
        .expect("LANGFUSE_SECRET_KEY must be set");
    let host = std::env::var("LANGFUSE_HOST")
        .unwrap_or_else(|_| "https://cloud.langfuse.com".to_string());
    
    // Create base64 encoded credentials for OTEL
    let credentials = base64::engine::general_purpose::STANDARD
        .encode(format!("{}:{}", public_key, secret_key));
    
    // Set OTEL environment variables with base endpoint (will append /v1/traces)
    std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", format!("{}/api/public", host));
    std::env::set_var("OTEL_EXPORTER_OTLP_HEADERS", format!("Authorization=Basic {}", credentials));
    
    println!("  OTEL_EXPORTER_OTLP_ENDPOINT={}/api/public", host);
    println!("  OTEL_EXPORTER_OTLP_HEADERS=Authorization=Basic <credentials>");
    println!("  (Will append /v1/traces to endpoint)");
    
    // Create exporter using OTEL environment variables
    let exporter = exporter_from_otel_env()?;
    
    // Create tracer provider
    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(Resource::builder().with_attributes(vec![
            KeyValue::new("service.name", "otel-env-test-base-endpoint"),
            KeyValue::new("test.type", "OTEL_EXPORTER_OTLP_ENDPOINT"),
        ]).build())
        .build();
    
    global::set_tracer_provider(provider.clone());
    
    // Create test trace
    let tracer = global::tracer("otel-test-base");
    let mut span = tracer
        .span_builder("test-otel-base-endpoint")
        .with_attributes(vec![
            KeyValue::new("test.endpoint_type", "base_endpoint"),
            KeyValue::new("test.config", "OTEL_EXPORTER_OTLP_ENDPOINT"),
        ])
        .start(&tracer);
    
    span.add_event("Testing OTEL_EXPORTER_OTLP_ENDPOINT configuration", vec![]);
    span.end();
    
    println!("  Trace created: test-otel-base-endpoint");
    
    // Give time for export
    sleep(Duration::from_secs(2)).await;
    drop(provider);
    sleep(Duration::from_secs(1)).await;
    
    // Verify in Langfuse
    println!("  Verifying trace in Langfuse...");
    // Set Langfuse env vars temporarily for the client
    std::env::set_var("LANGFUSE_HOST", &host);
    std::env::set_var("LANGFUSE_PUBLIC_KEY", &public_key);
    std::env::set_var("LANGFUSE_SECRET_KEY", &secret_key);
    let client = LangfuseClient::from_env()?;
    std::env::remove_var("LANGFUSE_HOST");
    std::env::remove_var("LANGFUSE_PUBLIC_KEY");
    std::env::remove_var("LANGFUSE_SECRET_KEY");
    
    let response = client.list_traces().limit(5).call().await?;
    let found = if let Some(data) = response.get("data") {
        if let Some(array) = data.as_array() {
            array.iter().any(|t| 
                t.get("name")
                    .and_then(|n| n.as_str())
                    .map(|n| n.contains("test-otel-base-endpoint"))
                    .unwrap_or(false)
            )
        } else {
            false
        }
    } else {
        false
    };
    
    if found {
        println!("  ✅ Trace found in Langfuse!");
    } else {
        println!("  ⚠️  Trace not found yet (may need more time to process)");
    }
    
    // Clean up
    std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
    std::env::remove_var("OTEL_EXPORTER_OTLP_HEADERS");
    
    Ok(())
}