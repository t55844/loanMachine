// tests/routes_integration_test.rs

#![cfg(not(target_arch = "wasm32"))]
mod common;
use common::{get_deployed, build_test_router, extract_and_decode};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt; // gives us .oneshot()
use serde_json::{json, Value};

const MEMBER1_ADDRESS: &str = "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC"; 
const MEMBER1_ID: u32       = 1;
const MEMBER2_ADDRESS: &str = "0x90F79bf6EB2c4f870365E785982E1f101E93b906";
const MEMBER2_ID: u32       = 2;
const ACCESS_CODE: &str     = "test123";


// helper: build a POST request with a JSON body
fn post_json(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

// helper: read response body bytes into a Value
async fn read_json(body: axum::body::Body) -> Value {
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

// ── validation ────────────────────────────────────────────────

#[tokio::test]
async fn rejects_invalid_wallet_address() {
    let router = build_test_router().await;

    let res = router
        .oneshot(post_json("/api/vinculate/prepare", json!({
            "member_id":      MEMBER1_ID,
            "wallet_address": "not-a-wallet",
            "coop_id":        get_deployed().coop_id,
            "access_code":    ACCESS_CODE,
        })))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rejects_invalid_coop_id() {
    let router = build_test_router().await;

    let res = router
        .oneshot(post_json("/api/vinculate/prepare", json!({
            "member_id":      MEMBER1_ID,
            "wallet_address": MEMBER1_ADDRESS,
            "coop_id":        "not-a-bytes32",
            "access_code":    ACCESS_CODE,
        })))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

// ── happy path ────────────────────────────────────────────────

#[tokio::test]
async fn prepare_vinculation_returns_calldata_and_gas() {
    let env    = get_deployed();
    let router = build_test_router().await;

    let res = router
        .oneshot(post_json("/api/vinculate/prepare", json!({
            "member_id":      MEMBER2_ID,
            "wallet_address": MEMBER2_ADDRESS,
            "coop_id":        env.coop_id,
            "access_code":    ACCESS_CODE,
        })))
        .await
        .unwrap();

    let status = res.status();
    
    // print body regardless of status so we can see the error
    let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
    let body_str   = String::from_utf8_lossy(&body_bytes);
    println!("STATUS: {status}");
    println!("BODY:   {body_str}");

    if status != StatusCode::OK {
            let decoded = extract_and_decode(&body_str);
            panic!(
                "STATUS: {status}\nBODY:   {body_str}\nDecoded revert: {decoded}"
            );
        }
    assert_eq!(status, StatusCode::OK);
}