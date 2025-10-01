//! Example demonstrating custom configuration options.
//!
//! This example shows various ways to configure the Langfuse exporter:
//! - Custom HTTP client with proxy
//! - Custom timeout settings
//! - Additional headers
//! - TLS configuration
//! - Manual configuration (not using environment variables)
//!
//! Run with:
//! ```bash
//! # For basic example (using env vars):
//! export LANGFUSE_PUBLIC_KEY="pk-lf-..."
//! export LANGFUSE_SECRET_KEY="sk-lf-..."
//! export LANGFUSE_HOST="https://cloud.langfuse.com"
//!
//! cargo run --example custom_config
//! ```

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use opentelemetry::global;
    use opentelemetry::trace::{Span, SpanKind, Tracer};
    use opentelemetry::KeyValue;
    use opentelemetry_langfuse::ExporterBuilder;
    use opentelemetry_sdk::trace::{SdkTracerProvider, SimpleSpanProcessor};
    use opentelemetry_sdk::Resource;
    use std::time::Duration;

    println!("=== Custom Configuration Examples ===\n");

    // Example 1: Using environment variables with default settings
    println!("1. Default configuration from environment:");
    let default_exporter = ExporterBuilder::from_env()?.build()?;
    println!("   ✓ Created exporter with default settings\n");

    // Example 2: Manual configuration without environment variables
    println!("2. Manual configuration:");
    let _manual_exporter = ExporterBuilder::new()
        .with_host("https://cloud.langfuse.com")
        .with_basic_auth("pk-lf-your-key", "sk-lf-your-secret")
        .with_timeout(Duration::from_secs(30))
        .build()?;
    println!("   ✓ Created exporter with explicit settings\n");

    // Example 3: Custom HTTP client with proxy (commented out - requires proxy)
    println!("3. Custom HTTP client example (see code):");
    /*
    let custom_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .proxy(reqwest::Proxy::http("http://proxy.example.com:8080")?)
        .pool_max_idle_per_host(10)
        .build()?;

    let proxy_exporter = ExporterBuilder::new()
        .with_host("https://cloud.langfuse.com")
        .with_basic_auth("pk-lf-...", "sk-lf-...")
        .with_http_client(custom_client)
        .build()?;
    */
    println!("   (Uncomment proxy code to test with your proxy)\n");

    // Example 4: Custom headers for additional metadata
    println!("4. Custom headers configuration:");
    let _headers_exporter = ExporterBuilder::from_env()? // Load from env first
        .with_header("X-Service-Version", "1.2.3")
        .with_header("X-Environment", "staging")
        .with_headers(vec![("X-Team", "backend"), ("X-Region", "us-west-2")])
        .build()?;
    println!("   ✓ Added custom headers for metadata\n");

    // Example 5: Native TLS configuration (requires native-tls feature in your app)
    println!("5. Native TLS example (see code):");
    /*
    // Add to your Cargo.toml:
    // reqwest = { version = "0.12", features = ["native-tls"] }

    let native_tls_client = reqwest::Client::builder()
        .use_native_tls()
        .build()?;

    let native_tls_exporter = ExporterBuilder::from_env()?
        .with_http_client(native_tls_client)
        .build()?;
    */
    println!("   (See code comments for native-tls setup)\n");

    // Example 6: Custom root certificates for self-hosted Langfuse
    println!("6. Custom certificates example (see code):");
    /*
    use std::fs;

    let cert = fs::read("path/to/ca-cert.pem")?;
    let cert = reqwest::Certificate::from_pem(&cert)?;

    let custom_ca_client = reqwest::Client::builder()
        .add_root_certificate(cert)
        .build()?;

    let self_hosted_exporter = ExporterBuilder::new()
        .with_host("https://langfuse.mycompany.com")
        .with_basic_auth("pk-lf-...", "sk-lf-...")
        .with_http_client(custom_ca_client)
        .build()?;
    */
    println!("   (See code comments for custom CA setup)\n");

    // Example 7: Complete production configuration
    println!("7. Production configuration with all options:");

    // This would typically come from your config system
    let production_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .pool_idle_timeout(Duration::from_secs(90))
        .pool_max_idle_per_host(100)
        .tcp_keepalive(Duration::from_secs(60))
        .build()?;

    let _production_exporter = ExporterBuilder::from_env()?  // Start with env vars
        .with_timeout(Duration::from_secs(45))  // Override timeout
        .with_header("X-Service-Name", "production-api")
        .with_http_client(production_client)
        .build()?;

    println!("   ✓ Created production-ready exporter\n");

    // Use one of the exporters with a tracer
    println!("=== Using the exporter ===\n");

    let provider = SdkTracerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_attributes([KeyValue::new("service.name", "config-example")])
                .build(),
        )
        .with_span_processor(SimpleSpanProcessor::new(default_exporter))
        .build();

    global::set_tracer_provider(provider.clone());
    let tracer = global::tracer("custom-config-demo");

    // Create a test span
    {
        let mut span = tracer
            .span_builder("test-configuration")
            .with_kind(SpanKind::Internal)
            .with_attributes([KeyValue::new("config.type", "custom")])
            .start(&tracer);

        std::thread::sleep(Duration::from_millis(10));
        span.set_attribute(KeyValue::new("test.passed", true));
        span.end();
    }

    drop(provider);

    println!("✅ Configuration examples completed!");
    println!("\nKey takeaways:");
    println!("  - Use environment variables for standard deployments");
    println!("  - Use ExporterBuilder for programmatic configuration");
    println!("  - Custom HTTP client for proxy/TLS/timeout control");
    println!("  - Add custom headers for additional metadata");
    println!("\nCheck your Langfuse dashboard for the test span.");

    Ok(())
}
