use leptos::prelude::*;
use crate::components::cooperatives::{CoopCard, CooperativesPage};
use loan_machine_models::responses::CooperativeView;
use leptos_router::components::Router;
use leptos_router::location::RequestUrl;

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
//
// The page is only ever mounted by RequireWallet's Connected arm, so
// we don't simulate "disconnected" here — that's impossible by type.
// What we verify is: in SSR, the LocalResource never resolves, so the
// page renders its <Suspense> fallback, and that fallback contains
// the section title + spinner. Resource-resolution behaviour belongs
// in wasm-bindgen-test integration tests, not here.

fn page_html() -> String {
    render_to_string(move || {
        provide_context(RequestUrl::new("/"));
        view! {
            <Router>
                <CooperativesPage />
            </Router>
        }
    })
}
#[test]
fn page_shows_section_title() {
    let html = page_html();
    assert!(html.contains("REGISTERED COOPERATIVES"),
        "expected section title, got: {html}");
}

#[test]
fn page_renders_suspense_fallback_during_initial_render() {
    let html = page_html();
    assert!(html.contains("spinner"), "expected spinner, got: {html}");
}

#[test]
fn page_does_not_show_error_on_initial_render() {
    let html = page_html();
    assert!(!html.contains("Failed to load"));
}

#[test]
fn page_does_not_show_empty_state_on_initial_render() {
    let html = page_html();
    assert!(!html.contains("No cooperatives registered yet"));
}

// ── CoopCard ─────────────────────────────────────────────────
// (unchanged — CoopCard never depended on wallet state)


fn card_html(coop: CooperativeView) -> String {
    render_to_string(move || {
        provide_context(RequestUrl::new("/"));
        view! {
            <Router>
                <CoopCard coop=coop />
            </Router>
        }
    })
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
    assert!(html.contains("1700000000"), "expected timestamp, got: {html}");
}