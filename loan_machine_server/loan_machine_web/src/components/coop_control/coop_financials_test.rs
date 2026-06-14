// SSR snapshot tests for the coop-wide money flow panel.
//
// `LocalResource` never resolves during SSR, so the initial render is
// just the Suspense fallback (spinner) — same caveat as
// `vinculation_test.rs`.

use leptos::prelude::*;
use crate::components::coop_control::coop_financials::CoopFinancialsPanel;

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
    render_to_string(|| view! { <CoopFinancialsPanel coop_id="0xabc" /> })
}

#[test]
fn panel_shows_loading_spinner_initially() {
    assert!(panel_html().contains("spinner"));
}

#[test]
fn panel_hides_money_flow_tag_initially() {
    assert!(!panel_html().contains("MONEY FLOW"));
}
