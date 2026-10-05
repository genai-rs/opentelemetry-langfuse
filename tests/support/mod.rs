use langfuse_ergonomic::LangfuseClient;
use serde_json::Value;
use std::time::Duration;
use tokio::time::{sleep, timeout};

pub struct ObservationQuery {
    pub trace_id: String,
    pub span_id: String,
    pub name: String,
    pub from_start_time: String,
    pub to_start_time: String,
}

/// Query only the exported observation using Langfuse Cloud/v4's read API.
/// The ergonomic client does not yet wrap v2, so reuse its authenticated transport.
pub async fn verify_observation(
    client: &LangfuseClient,
    query: &ObservationQuery,
    budget: Duration,
    interval: Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    timeout(budget, poll_observation(client, query, interval))
        .await
        .map_err(|_| format!("Observation {} not found within {budget:?}", query.span_id))?
}

async fn poll_observation(
    client: &LangfuseClient,
    query: &ObservationQuery,
    interval: Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = client.configuration();
    loop {
        let mut request = config
            .client
            .get(format!(
                "{}/api/public/v2/observations",
                config.base_path.trim_end_matches('/')
            ))
            .query(&[
                ("fields", "core,basic"),
                ("traceId", query.trace_id.as_str()),
                ("name", query.name.as_str()),
                ("limit", "1"),
                ("fromStartTime", query.from_start_time.as_str()),
                ("toStartTime", query.to_start_time.as_str()),
            ])
            .timeout(Duration::from_secs(10));
        if let Some((username, password)) = &config.basic_auth {
            request = request.basic_auth(username, password.as_ref());
        }
        if let Some(user_agent) = &config.user_agent {
            request = request.header(reqwest::header::USER_AGENT, user_agent);
        }

        let response = match request.send().await {
            Ok(response) => response,
            Err(_) => {
                sleep(interval).await;
                continue;
            }
        };
        let status = response.status();
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            // Langfuse sends Retry-After as seconds. Never retry before that window.
            let delay = response
                .headers()
                .get(reqwest::header::RETRY_AFTER)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.parse::<u64>().ok())
                .map(Duration::from_secs)
                .unwrap_or(Duration::from_secs(60));
            sleep(delay.max(interval)).await;
            continue;
        }
        if status.is_server_error() {
            sleep(interval).await;
            continue;
        }
        if !status.is_success() {
            return Err(format!("Langfuse observation query failed: HTTP {status}").into());
        }

        let body: Value = serde_json::from_str(&response.text().await?)?;
        let observations = body["data"]
            .as_array()
            .ok_or("Langfuse observation response is missing its data array")?;
        if observations.iter().any(|observation| {
            observation["traceId"].as_str() == Some(query.trace_id.as_str())
                && observation["id"].as_str() == Some(query.span_id.as_str())
                && observation["name"].as_str() == Some(query.name.as_str())
        }) {
            return Ok(());
        }
        sleep(interval).await;
    }
}
