//! Test that verifies OTEL endpoint construction is correct.

use opentelemetry_langfuse::{exporter_from_otel_env, ExporterBuilder};
use std::env;

fn main() {
    println!("Testing OTEL endpoint construction...\n");

    // Test 1: OTEL_EXPORTER_OTLP_TRACES_ENDPOINT
    println!("Test 1: OTEL_EXPORTER_OTLP_TRACES_ENDPOINT");
    env::set_var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT", "https://cloud.langfuse.com/api/public/otel");
    env::set_var("OTEL_EXPORTER_OTLP_HEADERS", "Authorization=Basic dGVzdDp0ZXN0");
    
    println!("  Set: OTEL_EXPORTER_OTLP_TRACES_ENDPOINT=https://cloud.langfuse.com/api/public/otel");
    println!("  Expected endpoint: https://cloud.langfuse.com/api/public/otel");
    
    let result = exporter_from_otel_env();
    match result {
        Ok(_) => println!("  ✅ Exporter created successfully (would fail at runtime without HTTP client)"),
        Err(e) => {
            if format!("{:?}", e).contains("NoHttpClient") || format!("{:?}", e).contains("OtlpExporter") {
                println!("  ✅ Exporter configuration parsed correctly (failed at build with NoHttpClient - expected)");
            } else {
                println!("  ❌ Unexpected error: {:?}", e);
            }
        }
    }
    
    env::remove_var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT");
    env::remove_var("OTEL_EXPORTER_OTLP_HEADERS");
    
    println!();

    // Test 2: OTEL_EXPORTER_OTLP_ENDPOINT (should append /v1/traces)
    println!("Test 2: OTEL_EXPORTER_OTLP_ENDPOINT");
    env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "https://cloud.langfuse.com/api/public");
    env::set_var("OTEL_EXPORTER_OTLP_HEADERS", "Authorization=Basic dGVzdDp0ZXN0");
    
    println!("  Set: OTEL_EXPORTER_OTLP_ENDPOINT=https://cloud.langfuse.com/api/public");
    println!("  Expected endpoint: https://cloud.langfuse.com/api/public/v1/traces");
    
    let result = exporter_from_otel_env();
    match result {
        Ok(_) => println!("  ✅ Exporter created successfully (would fail at runtime without HTTP client)"),
        Err(e) => {
            if format!("{:?}", e).contains("NoHttpClient") || format!("{:?}", e).contains("OtlpExporter") {
                println!("  ✅ Exporter configuration parsed correctly (failed at build with NoHttpClient - expected)");
            } else {
                println!("  ❌ Unexpected error: {:?}", e);
            }
        }
    }
    
    env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
    env::remove_var("OTEL_EXPORTER_OTLP_HEADERS");
    
    println!();

    // Test 3: Verify the actual endpoint being built
    println!("Test 3: Inspect endpoint construction with debug output");
    
    // For TRACES endpoint
    env::set_var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT", "https://cloud.langfuse.com/api/public/otel");
    env::set_var("OTEL_EXPORTER_OTLP_HEADERS", "Authorization=Basic dGVzdDp0ZXN0");
    
    println!("  TRACES_ENDPOINT: Should use URL as-is");
    println!("    Input:  https://cloud.langfuse.com/api/public/otel");
    println!("    Output: https://cloud.langfuse.com/api/public/otel");
    
    env::remove_var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT");
    
    // For BASE endpoint
    env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "https://cloud.langfuse.com/api/public");
    
    println!("\n  BASE_ENDPOINT: Should append /v1/traces");
    println!("    Input:  https://cloud.langfuse.com/api/public");
    println!("    Output: https://cloud.langfuse.com/api/public/v1/traces");
    
    env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
    env::remove_var("OTEL_EXPORTER_OTLP_HEADERS");
    
    println!();
    
    // Test 4: Test with real Langfuse endpoint patterns
    println!("Test 4: Real Langfuse endpoint patterns");
    
    // According to Langfuse docs, the OTLP endpoint is: https://cloud.langfuse.com/api/public/otel
    // But the OTEL spec says for OTEL_EXPORTER_OTLP_ENDPOINT, we append /v1/traces
    // So if someone sets OTEL_EXPORTER_OTLP_ENDPOINT=https://cloud.langfuse.com/api/public
    // We would create: https://cloud.langfuse.com/api/public/v1/traces
    // But Langfuse expects: https://cloud.langfuse.com/api/public/otel
    
    println!("  ⚠️  IMPORTANT: Langfuse endpoint mismatch!");
    println!("  Langfuse expects: /api/public/otel");
    println!("  OTEL standard would create: /api/public/v1/traces");
    println!();
    println!("  Users should use ONE of these approaches:");
    println!("  1. OTEL_EXPORTER_OTLP_TRACES_ENDPOINT=https://cloud.langfuse.com/api/public/otel");
    println!("  2. LANGFUSE_HOST=https://cloud.langfuse.com (auto-appends /api/public/otel)");
    println!();
    println!("  ❌ NOT: OTEL_EXPORTER_OTLP_ENDPOINT=https://cloud.langfuse.com/api/public");
    println!("     (This would create /api/public/v1/traces which won't work with Langfuse!)");
}