// loan_machine_web/src/components/loan_requisition/loan_requisition_test.rs
//
// SSR snapshot tests for the loan requisition form.
//
// Only the INITIAL RENDER is tested here. Reactivity (signal updates,
// events, Action futures) does not tick during SSR rendering.

use leptos::prelude::*;
use crate::components::loan_requisition::loan_requisition::LoanRequisitionForm;
use crate::components::gas_modal::provide_gas_modal;

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
        view! { <LoanRequisitionForm coop_id="0xabc" on_tx_success=|| {} /> }
    })
}

// ── structure ─────────────────────────────────────────────────

#[test]
fn form_shows_request_loan_tag() {
    assert!(form_html().contains("REQUEST LOAN"), "expected card tag");
}

#[test]
fn form_shows_amount_input() {
    assert!(form_html().contains("Amount (USDT)"), "expected amount label");
}

#[test]
fn form_shows_parcels_input() {
    assert!(
        form_html().contains("Number of installments"),
        "expected installments label",
    );
}

#[test]
fn form_shows_interval_input() {
    assert!(
        form_html().contains("Payment interval"),
        "expected interval label",
    );
}

#[test]
fn form_does_not_show_coverage_input() {
    let html = form_html();
    assert!(
        !html.to_lowercase().contains("minimum coverage"),
        "coverage input should not appear — it is hardcoded in the contract",
    );
}

// ── initial tx-status state ───────────────────────────────────

#[test]
fn form_hides_tx_status_initially() {
    let html = form_html();
    assert!(!html.contains("Waiting for signature"),          "pending state leaked");
    assert!(!html.contains("Loan requisition submitted"),     "complete state leaked");
    assert!(!html.contains("Failed to submit transaction"),   "failed state leaked");
}

