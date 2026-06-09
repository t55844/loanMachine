// loan_machine_web/src/components/vinculation_test.rs
//
// SSR snapshot tests for the vinculation components.
//
// These only cover the INITIAL RENDER of the form.  Reactivity
// (signal updates, events, Action futures) does not tick in SSR
// rendering — those cases need wasm-bindgen-test in a real browser.
// See `vinculation_wasm_test.rs` for that layer.
//
// Note: the old `VinculationGate` component was removed during the
// client-side refactor; gating is now done by `RequireWallet` in
// `app.rs`.  The gate tests went with it.

use leptos::prelude::*;
use crate::components::vinculation::FirstVinculationForm;
use crate::components::gas_modal::provide_gas_modal;
// ── Helpers ──────────────────────────────────────────────────

/// Render any view to an HTML string inside an Owner scope.
/// The Owner is required because signals register with it at creation.
fn render_to_string<V, F>(f: F) -> String
where
    F: FnOnce() -> V,
    V: IntoView,
{
    let owner = Owner::new();
    let html = owner.with(|| f().to_html());
    drop(owner);
    html
}

// ── FirstVinculationForm ─────────────────────────────────────
//
// Fully synchronous on mount — no spawn_local, no Action dispatch
// until the user submits — so SSR render = real first paint.

fn form_html() -> String {
    render_to_string(|| {
        provide_gas_modal();
        view! {<FirstVinculationForm on_success=|| {} />}
    })
}
// Title + subtitle
#[test]
fn form_shows_title() {
    assert!(form_html().contains("LINK YOUR WALLET"));
}

#[test]
fn form_shows_updated_subtitle() {
    // Text was changed from "ID de membro" to "documento"
    assert!(form_html().contains("Connect your document"));
}

// Card 1 — identification
#[test]
fn form_shows_step_01_card() {
    assert!(form_html().contains("STEP 01"));
}

#[test]
fn form_shows_both_doc_kind_toggles() {
    let html = form_html();
    assert!(html.contains(">CPF<"),  "CPF toggle button missing");
    assert!(html.contains(">CNPJ<"), "CNPJ toggle button missing");
}

#[test]
fn form_shows_coop_id_input() {
    assert!(form_html().contains("Cooperative ID"));
}

#[test]
fn form_shows_access_code_input() {
    assert!(form_html().contains("Access Code"));
}

#[test]
fn form_shows_prepare_button() {
    assert!(form_html().contains("PREPARE LINKING"));
}

#[test]
fn form_hides_tx_status_initially() {
    let html = form_html();
    assert!(!html.contains("Waiting for signature"),     "pending state leaked");
    assert!(!html.contains("LINKING SUBMITTED"),         "complete state leaked");
    assert!(!html.contains("Failed to submit"),          "failed state leaked");
}

