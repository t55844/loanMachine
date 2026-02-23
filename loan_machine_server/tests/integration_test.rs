use std::sync::Arc;
use loan_machine_server::config::AppState;
use loan_machine_server::create_app;
use serde_json::json;
use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
};

use tower::ServiceExt;

#[tokio::test]
async fn test_vinculate_member_preparation() {
    let state = Arc::new(AppState::new().await);
    let app = create_app(state);

    // Use a fresh address that hasn't been linked yet
    let payload = json!({
        "member_id": 99,
        "wallet_address": "0xbDA5747bFD65F08deb54cb465eB87D40e51B197E" // Hardhat account #2
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/vinculate/prepare")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify we got transaction data back
    assert!(body.get("data").unwrap().as_str().unwrap().starts_with("0x"));
}