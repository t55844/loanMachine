// loan_machine_core/src/services/subgraph/tests.rs
//
// Tests for SubgraphService. Strategy: spin up a real local HTTP server
// with `wiremock`, point SubgraphService at it, and assert behavior.
// The URL is the seam — no trait abstraction needed.
//
// What this covers:
//   1. Happy path: typed deserialization of a populated `data` field.
//   2. Empty list: `{"data": {"donatedEvents": []}}` → empty Vec.
//   3. GraphQL errors: 200 OK with `errors` array → SubgraphError::Graphql.
//   4. HTTP 5xx: → SubgraphError::Network.
//   5. Malformed body: 200 OK with non-JSON → SubgraphError::Decode.
//   6. Missing `data`: 200 OK with `{}` → SubgraphError::MissingData.
//   7. Outgoing request shape: server fn built `query` + `variables` correctly.

use loan_machine_core::services::subgraph::{SubgraphError, SubgraphService};
use serde::{Deserialize, Serialize};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// --- Test fixtures: stand-ins for what server_logic would define ----------

#[derive(Debug, Deserialize, PartialEq)]
struct DonatedEvent {
    id: String,
    donor: String,
    amount: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct EventsData {
    #[serde(rename = "donatedEvents")]
    donated_events: Vec<DonatedEvent>,
}

#[derive(Serialize)]
struct EmptyVars {}

const QUERY: &str = "query { donatedEvents { id donor amount } }";

// --- 1. Happy path --------------------------------------------------------

#[tokio::test]
async fn returns_typed_data_on_success() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "donatedEvents": [
                    {"id": "1", "donor": "0xaaa", "amount": "100"},
                    {"id": "2", "donor": "0xbbb", "amount": "250"}
                ]
            }
        })))
        .mount(&server)
        .await;

    let svc = SubgraphService::new(server.uri());
    let data: EventsData = svc.query(QUERY, EmptyVars {}).await.unwrap();

    assert_eq!(data.donated_events.len(), 2);
    assert_eq!(data.donated_events[0].id, "1");
    assert_eq!(data.donated_events[0].donor, "0xaaa");
    assert_eq!(data.donated_events[0].amount, "100");
    assert_eq!(data.donated_events[1].id, "2");
}

// --- 2. Empty list --------------------------------------------------------

#[tokio::test]
async fn returns_empty_vec_when_data_is_empty() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": { "donatedEvents": [] }
        })))
        .mount(&server)
        .await;

    let svc = SubgraphService::new(server.uri());
    let data: EventsData = svc.query(QUERY, EmptyVars {}).await.unwrap();

    assert!(data.donated_events.is_empty());
}

// --- 3. GraphQL-level errors ---------------------------------------------

#[tokio::test]
async fn graphql_errors_become_graphql_variant() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "errors": [
                {"message": "Field 'foo' not defined on DonatedEvent"},
                {"message": "Variable $bar is undeclared"}
            ]
        })))
        .mount(&server)
        .await;

    let svc = SubgraphService::new(server.uri());
    let result: Result<EventsData, _> = svc.query(QUERY, EmptyVars {}).await;

    match result {
        Err(SubgraphError::Graphql(msg)) => {
            assert!(msg.contains("Field 'foo' not defined"));
            assert!(msg.contains("Variable $bar is undeclared"));
        }
        other => panic!("expected SubgraphError::Graphql, got {other:?}"),
    }
}

// --- 4. HTTP transport error ---------------------------------------------

#[tokio::test]
async fn http_5xx_becomes_network_variant() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let svc = SubgraphService::new(server.uri());
    let result: Result<EventsData, _> = svc.query(QUERY, EmptyVars {}).await;

    assert!(
        matches!(result, Err(SubgraphError::Network(_))),
        "expected SubgraphError::Network, got {result:?}"
    );
}

// --- 5. Malformed response body ------------------------------------------

#[tokio::test]
async fn malformed_json_becomes_decode_variant() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json at all"))
        .mount(&server)
        .await;

    let svc = SubgraphService::new(server.uri());
    let result: Result<EventsData, _> = svc.query(QUERY, EmptyVars {}).await;

    assert!(
        matches!(result, Err(SubgraphError::Decode(_))),
        "expected SubgraphError::Decode, got {result:?}"
    );
}

// --- 6. Missing data field -----------------------------------------------

#[tokio::test]
async fn missing_data_becomes_missing_data_variant() {
    let server = MockServer::start().await;

    // Valid JSON, no `data`, no `errors`. Pathological but possible.
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .mount(&server)
        .await;

    let svc = SubgraphService::new(server.uri());
    let result: Result<EventsData, _> = svc.query(QUERY, EmptyVars {}).await;

    assert!(
        matches!(result, Err(SubgraphError::MissingData)),
        "expected SubgraphError::MissingData, got {result:?}"
    );
}

// --- 7. Outgoing request body contract -----------------------------------

#[tokio::test]
async fn outgoing_request_carries_query_and_variables() {
    let server = MockServer::start().await;

    #[derive(Serialize)]
    struct Vars {
        borrower: String,
        first: u32,
    }

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": { "donatedEvents": [] }
        })))
        .mount(&server)
        .await;

    let svc = SubgraphService::new(server.uri());
    let q = "query Q($borrower: Bytes!, $first: Int!) { \
             donatedEvents(where: {donor: $borrower}, first: $first) \
             { id donor amount } }";

    let _: EventsData = svc
        .query(
            q,
            Vars {
                borrower: "0xdeadbeef".into(),
                first: 25,
            },
        )
        .await
        .unwrap();

    let received = server.received_requests().await.unwrap();
    assert_eq!(received.len(), 1, "expected exactly one request");

    let body: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(body["variables"]["borrower"], "0xdeadbeef");
    assert_eq!(body["variables"]["first"], 25);
    assert!(
        body["query"].as_str().unwrap().contains("donatedEvents"),
        "request body should carry the query string"
    );
}