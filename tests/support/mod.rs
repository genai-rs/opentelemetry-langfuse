use jiff::Timestamp;
use langfuse_ergonomic::{Error, LangfuseClient};
use std::time::Duration;
use tokio::time::{sleep, timeout};

pub struct ObservationQuery {
    pub trace_id: String,
    pub span_id: String,
    pub name: String,
    pub from_start_time: Timestamp,
    pub to_start_time: Timestamp,
}

/// Wait until Langfuse returns the exported observation, through the Observations API v2
/// (the Cloud/v4 read API) of langfuse-ergonomic.
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
    loop {
        let page = client
            .observations()
            .fields("core,basic")
            .trace_id(query.trace_id.as_str())
            .name(query.name.as_str())
            .limit(1)
            .from_start_time(query.from_start_time)
            .to_start_time(query.to_start_time)
            .call()
            .await;
        let page = match page {
            Ok(page) => page,
            // Langfuse sends Retry-After as seconds. Never retry before that window.
            Err(Error::RateLimit { retry_after, .. }) => {
                sleep(retry_after.unwrap_or(Duration::from_secs(60)).max(interval)).await;
                continue;
            }
            Err(error) if error.is_retryable() => {
                sleep(interval).await;
                continue;
            }
            Err(error) => return Err(format!("Langfuse observation query failed: {error}").into()),
        };
        if page.data.iter().any(|observation| {
            observation.trace_id.as_deref() == Some(query.trace_id.as_str())
                && observation.id == query.span_id
                && observation.name.as_ref().and_then(Option::as_deref) == Some(query.name.as_str())
        }) {
            return Ok(());
        }
        sleep(interval).await;
    }
}
