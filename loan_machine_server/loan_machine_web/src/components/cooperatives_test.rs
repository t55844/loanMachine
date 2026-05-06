// SSR snapshot tests for cooperatives components.
//
// Tier 1 only: initial render, no reactivity, no spawn_local resolution.
// The LocalResource inside CooperativesPage never resolves in SSR,
// so the page always renders the Suspense fallback — that's the
// behaviour these tests verify.

use leptos::prelude::*;
use crate::components::cooperatives::{CooperativesPage, CoopCard};
use loan_machine_models::responses::CooperativeView;

// ── helpers ──────────────────────────────────────────────────

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

fn fixture_active() -> CooperativeView {
    CooperativeView {
        id: "0xabc123".to_string(),
        coop_id: "0xdeadbeef".to_string(),
        name: "Cooperativa Teste Ativa".to_string(),
        loan_machine: "0x1111111111111111111111111111111111111111".to_string(),
        active: true,
        registered_at: 1_700_000_000,
    }
}

fn fixture_inactive() -> CooperativeView {
    CooperativeView {
        id: "0xdef456".to_string(),
        coop_id: "0xcafebabe".to_string(),
        name: "Cooperativa Teste Inativa".to_string(),
        loan_machine: "0x2222222222222222222222222222222222222222".to_string(),
        active: false,
        registered_at: 1_700_000_500,
    }
}

// ── CooperativesPage ─────────────────────────────────────────

fn page_html_with(wallet_value: Option<String>) -> String {
    render_to_string(move || {
        let (wallet, _set_wallet) = signal::<Option<String>>(wallet_value);
        view! { <CooperativesPage wallet=wallet /> }
    })
}

#[test]
fn page_shows_section_title() {
    let html = page_html_with(None);
    assert!(html.contains("REGISTERED COOPERATIVES"),
        "expected section title, got: {html}");
}

#[test]
fn page_renders_suspense_fallback_when_disconnected() {
    let html = page_html_with(None);
    // LocalResource never resolves in SSR -> fallback (LoadingState) renders.
    assert!(html.contains("spinner"), "expected spinner, got: {html}");
}

#[test]
fn page_renders_suspense_fallback_when_connected_too() {
    // Wallet present -> still SSR -> still fallback.
    let html = page_html_with(Some("0xabc".to_string()));
    assert!(html.contains("spinner"));
}

#[test]
fn page_does_not_show_error_on_initial_render() {
    let html = page_html_with(None);
    assert!(!html.contains("Failed to load"));
}

#[test]
fn page_does_not_show_empty_state_on_initial_render() {
    let html = page_html_with(None);
    assert!(!html.contains("No cooperatives registered yet"));
}

// ── CoopCard ─────────────────────────────────────────────────

fn card_html(coop: CooperativeView) -> String {
    render_to_string(move || view! { <CoopCard coop=coop /> })
}

#[test]
fn card_shows_cooperative_name() {
    let html = card_html(fixture_active());
    assert!(html.contains("Cooperativa Teste Ativa"));
}

#[test]
fn card_shows_active_badge_when_active() {
    let html = card_html(fixture_active());
    assert!(html.contains("ACTIVE"));
    assert!(!html.contains("INACTIVE"));
}

#[test]
fn card_shows_inactive_badge_when_inactive() {
    let html = card_html(fixture_inactive());
    assert!(html.contains("INACTIVE"));
}

#[test]
fn card_shows_cooperative_tag() {
    let html = card_html(fixture_active());
    assert!(html.contains("COOPERATIVE"));
}

#[test]
fn card_shows_loan_machine_address() {
    let html = card_html(fixture_active());
    assert!(html.contains("0x1111111111111111111111111111111111111111"));
}

#[test]
fn card_shows_coop_id() {
    let html = card_html(fixture_active());
    assert!(html.contains("0xdeadbeef"));
}

#[test]
fn card_shows_registered_timestamp() {
    let html = card_html(fixture_active());
    assert!(html.contains("1700000000"),
        "expected timestamp, got: {html}");
}