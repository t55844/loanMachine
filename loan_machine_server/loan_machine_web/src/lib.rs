// src/lib.rs
#![recursion_limit = "256"]
pub mod app;
pub mod components;
pub mod server_fns;

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
