// loan_machine_web/src/components/donation/donation_test.rs
//
// SSR snapshot tests for the donation form.
//
// These only cover the INITIAL RENDER of the form.  Reactivity
// (signal updates, events, Action futures) does not tick in SSR
// rendering — those cases need wasm-bindgen-test in a real browser.

use leptos::prelude::*;
use crate::components::donation::donation::DonationForm;
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

fn form_html() -> String {
    render_to_string(|| {
        provide_gas_modal();
        view! {<DonationForm coop_id="0xabc" on_tx_success=|| {} />}
    })
}

// ── DonationForm ──────────────────────────────────────────────

#[test]
fn form_shows_donate_tag() {
    assert!(form_html().contains("DONATE"));
}

#[test]
fn form_shows_amount_input() {
    assert!(form_html().contains("Amount (USDT)"));
}

#[test]
fn form_hides_tx_status_initially() {
    let html = form_html();
    assert!(!html.contains("Waiting for approval signature"), "pending state leaked");
    assert!(!html.contains("Donation submitted successfully"), "complete state leaked");
    assert!(!html.contains("Failed to submit"),                "failed state leaked");
}
