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
        from_start_time: "2026-10-05T12:00:00+00:00".into(),
        to_start_time: "2026-10-05T12:02:00+00:00".into(),
    }
}

fn matching_body() -> String {
    serde_json::json!({
        "data": [{
            "traceId": query().trace_id,
            "id": query().span_id,
            "name": query().name,
        }],
        "meta": {"cursor": null}
    })
    .to_string()
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
            Matcher::UrlEncoded("fromStartTime".into(), query().from_start_time),
            Matcher::UrlEncoded("toStartTime".into(), query().to_start_time),
        ]))
        .with_body_from_request(move |_| {
            match calls.fetch_add(1, Ordering::SeqCst) {
                0 => br#"{"data":[]}"#.to_vec(),
                1 => {
                    // A similarly named span from another trace must not pass.
                    serde_json::json!({"data": [{
                        "traceId": "other-trace", "id": query().span_id, "name": query().name
                    }]})
                    .to_string()
                    .into_bytes()
                }
                2 => serde_json::json!({"data": [{
                    "traceId": query().trace_id, "id": "other-span", "name": query().name
                }]})
                .to_string()
                .into_bytes(),
                _ => matching_body().into_bytes(),
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
    assert!(error.to_string().contains("401"));
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
