// src/lib.rs
#![cfg(feature = "ssr")]  // add this at the top of blockchain.rs
// Public exports that are needed by both server and client
pub mod config;
pub mod models;
pub mod services;
pub mod app;
pub mod components;

// Re-export the root component for convenience
pub use app::App;

// Server-only code: only compiled when the "ssr" feature is enabled
#[cfg(feature = "ssr")]
pub mod routes;

#[cfg(feature = "ssr")]
use axum::{Router, routing::post};
#[cfg(feature = "ssr")]
use tower_http::cors::CorsLayer;
#[cfg(feature = "ssr")]
use crate::config::AppState;
#[cfg(feature = "ssr")]
use crate::routes::{prepare_donation, prepare_vinculation_to_wallet};
#[cfg(feature = "ssr")]
use tower_http::services::ServeDir;

#[cfg(feature = "ssr")]
pub fn create_app() -> Router<AppState> {
    Router::new()
        .route("/api/vinculate/prepare", post(prepare_vinculation_to_wallet))
        .route("/api/donate/prepare", post(prepare_donation))
        .nest_service("/style", ServeDir::new("target/site/style"))  
        .layer(CorsLayer::permissive())
}