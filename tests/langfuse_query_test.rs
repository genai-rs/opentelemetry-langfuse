//! Integration tests for querying traces back from Langfuse.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod support;

use langfuse_ergonomic::ClientBuilder;
use mockito::{Matcher, Server};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use support::{verify_observation, ObservationQuery};

fn query() -> ObservationQuery {
    ObservationQuery {
        trace_id: "0123456789abcdef0123456789abcdef".into(),
        span_id: "0123456789abcdef".into(),
        name: "test-span".into(),
        from_start_time: "2026-10-05T12:00:00Z".parse().unwrap(),
        to_start_time: "2026-10-05T12:02:00Z".parse().unwrap(),
    }
}

/// An observation as the Observations API v2 returns it, with the fields it always sends.
fn observation(trace_id: &str, id: &str, name: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "traceId": trace_id,
        "name": name,
        "type": "SPAN",
        "projectId": "project",
        "startTime": "2026-10-05T12:01:00.000Z",
        "endTime": null,
        "parentObservationId": null,
        "modelId": null,
        "inputPrice": null,
        "outputPrice": null,
        "totalPrice": null,
    })
}

/// A page of the Observations API v2.
fn page(observations: Vec<serde_json::Value>) -> Vec<u8> {
    serde_json::json!({"data": observations, "meta": {"cursor": null}})
        .to_string()
        .into_bytes()
}

fn matching_body() -> Vec<u8> {
    page(vec![observation(
        &query().trace_id,
        &query().span_id,
        &query().name,
    )])
}

fn client(server: &Server) -> langfuse_ergonomic::LangfuseClient {
    ClientBuilder::new()
        .public_key("public")
        .secret_key("secret")
        .base_url(server.url())
        .build()
        .unwrap()
}

#[tokio::test]
async fn polls_bounded_exact_observation_until_it_arrives() {
    let mut server = Server::new_async().await;
    let calls = AtomicUsize::new(0);
    let mock = server
        .mock("GET", "/api/public/v2/observations")
        .match_header("authorization", "Basic cHVibGljOnNlY3JldA==")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("fields".into(), "core,basic".into()),
            Matcher::UrlEncoded("traceId".into(), query().trace_id),
            Matcher::UrlEncoded("name".into(), query().name),
            Matcher::UrlEncoded("limit".into(), "1".into()),
            Matcher::UrlEncoded("fromStartTime".into(), "2026-10-05T12:00:00Z".into()),
            Matcher::UrlEncoded("toStartTime".into(), "2026-10-05T12:02:00Z".into()),
        ]))
        .with_body_from_request(move |_| {
            match calls.fetch_add(1, Ordering::SeqCst) {
                0 => page(vec![]),
                // A similarly named span from another trace must not pass.
                1 => page(vec![observation(
                    "other-trace",
                    &query().span_id,
                    &query().name,
                )]),
                2 => page(vec![observation(
                    &query().trace_id,
                    "other-span",
                    &query().name,
                )]),
                _ => matching_body(),
            }
        })
        .expect(4)
        .create_async()
        .await;

    verify_observation(
        &client(&server),
        &query(),
        Duration::from_secs(2),
        Duration::from_millis(10),
    )
    .await
    .unwrap();
    mock.assert_async().await;
}

#[tokio::test]
async fn honors_retry_after_and_recovers_from_rate_limiting() {
    let mut server = Server::new_async().await;
    let limited = server
        .mock("GET", "/api/public/v2/observations")
        .match_query(Matcher::Any)
        .with_status(429)
        .with_header("Retry-After", "1")
        .expect(1)
        .create_async()
        .await;
    let success = server
        .mock("GET", "/api/public/v2/observations")
        .match_query(Matcher::Any)
        .with_body(matching_body())
        .expect(1)
        .create_async()
        .await;
    let started = Instant::now();
    verify_observation(
        &client(&server),
        &query(),
        Duration::from_secs(3),
        Duration::from_millis(10),
    )
    .await
    .unwrap();
    assert!(started.elapsed() >= Duration::from_secs(1));
    limited.assert_async().await;
    success.assert_async().await;
}

#[tokio::test]
async fn stops_on_authentication_failure_without_retrying() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/api/public/v2/observations")
        .match_query(Matcher::Any)
        .with_status(401)
        .expect(1)
        .create_async()
        .await;
    let error = verify_observation(
        &client(&server),
        &query(),
        Duration::from_secs(1),
        Duration::from_millis(10),
    )
    .await
    .unwrap_err();
    assert!(
        error.to_string().contains("Authentication failed"),
        "{error}"
    );
    mock.assert_async().await;
}

#[tokio::test]
async fn retry_after_cannot_exceed_the_total_polling_budget() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/api/public/v2/observations")
        .match_query(Matcher::Any)
        .with_status(429)
        .with_header("Retry-After", "3600")
        .expect(1)
        .create_async()
        .await;
    let error = verify_observation(
        &client(&server),
        &query(),
        Duration::from_millis(250),
        Duration::from_millis(10),
    )
    .await
    .unwrap_err();
    assert!(error.to_string().contains("not found within"));
    mock.assert_async().await;
}
