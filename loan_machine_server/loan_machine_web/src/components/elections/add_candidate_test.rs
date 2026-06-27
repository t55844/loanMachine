// SSR snapshot tests for the AddCandidate component.
//
// `Action` never dispatches and `Effect` callbacks never fire during SSR, so
// the rendered HTML is the static shell of the form — the wallet input, hint,
// button, and an empty error slot.  Tests assert on that static structure.

use leptos::prelude::*;
use crate::components::elections::add_candidate::AddCandidate;
use crate::components::gas_modal::provide_gas_modal;

fn render_to_string<V, F>(f: F) -> String
where
    F: FnOnce() -> V,
    V: IntoView,
{
    let owner = Owner::new();
    let html  = owner.with(|| f().to_html());
    drop(owner);
    html
}

fn component_html() -> String {
    render_to_string(|| {
        provide_gas_modal();
        view! {
            <AddCandidate
                coop_id="0xabc"
                election_id=1u32
                on_tx_success=Callback::new(|()| {})
            />
        }
    })
}

// ── static shell ──────────────────────────────────────────────────────────────

#[test]
fn add_candidate_renders_add_candidate_button() {
    assert!(
        component_html().contains("ADD CANDIDATE"),
        "expected button label 'ADD CANDIDATE'",
    );
}

#[test]
fn add_candidate_renders_wallet_label() {
    assert!(
        component_html().contains("Add Candidate"),
        "expected input label containing 'Add Candidate'",
    );
}

#[test]
fn add_candidate_renders_wallet_placeholder() {
    assert!(
        component_html().contains("0x0000...0000"),
        "expected wallet placeholder '0x0000...0000'",
    );
}

#[test]
fn add_candidate_renders_hint_text() {
    assert!(
        component_html().contains("not already a candidate"),
        "expected hint about the candidate eligibility requirement",
    );
}

#[test]
fn add_candidate_renders_inside_form_group() {
    assert!(
        component_html().contains("form-group"),
        "expected outer div to carry the 'form-group' CSS class",
    );
}

// ── initial state (no error, no spinner, no tx status) ───────────────────────

#[test]
fn add_candidate_does_not_show_error_initially() {
    // The error alert is only injected by the reactive Effect after a failed
    // server call — never present in the initial SSR output.
    assert!(
        !component_html().contains("alert-error"),
        "error alert must not appear in the initial SSR render",
    );
}

#[test]
fn add_candidate_does_not_show_spinner_initially() {
    // The spinner only appears while the Action is pending (client-side only).
    assert!(
        !component_html().contains("spinner"),
        "spinner must not appear in the initial SSR render",
    );
}
