// src/lib.rs
pub mod app;
pub mod components;
pub mod models;
pub mod config;

#[cfg(feature = "ssr")]
pub mod services;

#[cfg(feature = "ssr")]
pub mod routes;

pub use app::App;

// ── WASM HYDRATION ENTRY POINT ────────────────────────────────
// This is what the browser calls after downloading the WASM bundle.
// HydrationScripts injects the JS that calls `mod.hydrate()`.
// Without this export, you get "mod.hydrate is not a function".
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    leptos::mount::hydrate_body(App);
}

// ── SSR ───────────────────────────────────────────────────────
#[cfg(feature = "ssr")]
use axum::{Router, routing::post};
#[cfg(feature = "ssr")]
use tower_http::cors::CorsLayer;
#[cfg(feature = "ssr")]
use crate::config::AppState;
#[cfg(feature = "ssr")]
use crate::routes::{prepare_donation, prepare_vinculation_to_wallet};

#[cfg(feature = "ssr")]
pub fn create_app() -> Router<AppState> {
    Router::new()
        .route("/api/vinculate/prepare", post(prepare_vinculation_to_wallet))
        .route("/api/donate/prepare", post(prepare_donation))
        .layer(CorsLayer::permissive())
}