//! Wiremock helpers for tests that point a `SubgraphService` at an
//! in-process HTTP mock. All helpers POST to `/` because that's what
//! `SubgraphService::new(uri)` targets.

use serde_json::Value;
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// One POST handler returning the same JSON for every request. Use when
/// the test doesn't need to distinguish between query names.
pub async fn server_returning(body: Value) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;
    server
}

/// One POST handler returning the given HTTP status with an empty body.
/// Use for error-path tests (500, 503, ...).
pub async fn server_returning_status(status: u16) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(status))
        .mount(&server)
        .await;
    server
}

/// Mount a POST handler that fires only when the request body contains
/// `query_marker`. Call once per query a test cares about; wiremock will
/// route by body match. Pair with `MockServer::start().await` directly
/// (not `server_returning`) when the test needs multiple distinct queries.
pub async fn mount_query(server: &MockServer, query_marker: &str, body: Value) {
    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_string_contains(query_marker))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(server)
        .await;
}