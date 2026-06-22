// SSR snapshot tests for the MyRequisitions component.
//
// LocalResource does not resolve during SSR, so Suspense always renders its
// fallback. These tests cover the static shell: the card tag and the spinner.

use leptos::prelude::*;
use crate::components::loan_requisition::my_requisitions::MyRequisitions;
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

fn panel_html() -> String {
    render_to_string(|| {
        provide_gas_modal();
        view! { <MyRequisitions coop_id="0xabc" /> }
    })
}

// ── structure ─────────────────────────────────────────────────

#[test]
fn panel_shows_my_loan_requisitions_tag() {
    assert!(
        panel_html().contains("MY LOAN REQUISITIONS"),
        "expected card tag 'MY LOAN REQUISITIONS'",
    );
}

#[test]
fn panel_shows_spinner_while_loading() {
    assert!(
        panel_html().contains("spinner"),
        "expected Suspense fallback to contain spinner",
    );
}

#[test]
fn panel_does_not_show_cancel_button_initially() {
    assert!(
        !panel_html().contains("CANCEL"),
        "cancel button should not appear in SSR — data is not yet resolved",
    );
}

#[test]
fn panel_does_not_show_requisition_data_initially() {
    let html = panel_html();
    // These strings only appear when the loaded list is rendered.
    assert!(!html.contains("USDT"),    "amount should not appear before data loads");
    assert!(!html.contains("parcels"), "parcel count should not appear before data loads");
}
