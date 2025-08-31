use opentelemetry_langfuse::ExporterBuilder;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Example 1: Using default HTTP client");
    let result = ExporterBuilder::new()
        .with_host("https://cloud.langfuse.com")
        .with_basic_auth("test-public-key", "test-secret-key")
        .build();

    match result {
        Ok(_) => println!("✅ Exporter with default client created successfully!"),
        Err(e) => println!("❌ Failed to create exporter: {}", e),
    }

    println!("\nExample 2: Using custom HTTP client with timeout");
    let custom_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .connection_verbose(true)
        .build()?;

    let result = ExporterBuilder::new()
        .with_host("https://cloud.langfuse.com")
        .with_basic_auth("test-public-key", "test-secret-key")
        .with_http_client(custom_client)
        .build();

    match result {
        Ok(_) => println!("✅ Exporter with custom client created successfully!"),
        Err(e) => println!("❌ Failed to create exporter: {}", e),
    }

    println!("\nExample 3: Using custom HTTP client with proxy");
    let proxy_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .proxy(reqwest::Proxy::http("http://proxy.example.com:8080")?)
        .build()?;

    let result = ExporterBuilder::new()
        .with_host("https://cloud.langfuse.com")
        .with_basic_auth("test-public-key", "test-secret-key")
        .with_http_client(proxy_client)
        .build();

    match result {
        Ok(_) => println!("✅ Exporter with proxy client created successfully!"),
        Err(e) => println!("❌ Failed to create exporter: {}", e),
    }

    Ok(())
}
