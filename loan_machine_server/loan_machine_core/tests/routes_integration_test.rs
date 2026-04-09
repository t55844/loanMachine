#![cfg(not(target_arch = "wasm32"))]
mod common;
use common::*;

use axum_test::TestServer;
use serde_json::json;
use loan_machine_core::create_app;

fn test_server() -> TestServer {
    let state = build_app_state(PLATFORM_ADMIN_KEY);
    let app   = create_app().with_state(state);
    TestServer::new(app).unwrap()
}

// ── validation: bad inputs ────────────────────────────────────

#[tokio::test]
async fn rejects_invalid_wallet_address() {
    let server = test_server();

    let res = server
        .post("/api/vinculate/prepare")
        .json(&json!({
            "member_id":      MEMBER1_ID,
            "wallet_address": "not-a-wallet",
            "coop_id":        COOP_ID,
            "access_code":    ACCESS_CODE,
        }))
        .await;

    assert_eq!(res.status_code(), 400);
}

#[tokio::test]
async fn rejects_invalid_coop_id() {
    let server = test_server();

    let res = server
        .post("/api/vinculate/prepare")
        .json(&json!({
            "member_id":      MEMBER1_ID,
            "wallet_address": MEMBER1_ADDRESS,
            "coop_id":        "not-a-bytes32",
            "access_code":    ACCESS_CODE,
        }))
        .await;

    assert_eq!(res.status_code(), 400);
}

// ── happy path ────────────────────────────────────────────────

#[tokio::test]
async fn prepare_vinculation_returns_calldata_and_gas() {
    let server = test_server();

    let res = server
        .post("/api/vinculate/prepare")
        .json(&json!({
            "member_id":      MEMBER1_ID,
            "wallet_address": MEMBER1_ADDRESS,
            "coop_id":        COOP_ID,
            "access_code":    ACCESS_CODE,
        }))
        .await;

    assert_eq!(res.status_code(), 200);

    let body: serde_json::Value = res.json();
    assert!(body["data"].as_str().unwrap().starts_with("0x"));
    assert!(body["gas_estimate"].as_str().unwrap() != "0");
    assert_eq!(
        body["to"].as_str().unwrap().to_lowercase(),
        LOAN_MACHINE_ADDR.to_lowercase()
    );
}