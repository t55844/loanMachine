// loan_machine_web/src/components/coop_control_panel_test.rs

use leptos::prelude::*;
use leptos_router::components::Router;
use leptos_router::location::RequestUrl;

use crate::components::coop_control::coop_control_panel::CoopControlPanelRoute;
use crate::components::tests_helper::seed_prices;
use crate::wallet_auth::session::{WalletCtx, WalletSession};

// ── Helper ────────────────────────────────────────────────

fn render_to_string<V: IntoView>(f: impl FnOnce() -> V) -> String {
    let owner = Owner::new();
    let html = owner.with(|| f().to_html());
    drop(owner);
    html
}

/// Full env for the panel: Router (params), RequestUrl (SSR), prices (BalanceRow
/// elsewhere in the page tree might consume it; defensive seed).

fn render_panel_disconnected() -> String {
    render_to_string(|| {
        provide_context(RequestUrl::new("/cooperatives/0xdeadbeef"));
        seed_prices(None);

        // RequireWallet needs WalletCtx in scope; Disconnected is the
        // simplest session that lets us assert "no role panel leaks."
        let (session, set_session) = signal(WalletSession::Disconnected);
        provide_context(WalletCtx { session, set: set_session });

        view! {
            <Router>
                <CoopControlPanelRoute />
            </Router>
        }
    })
}

// ── Suspense fallback path ────────────────────────────────
//
// The route mounts under RequireWallet. Without a WalletCtx in scope,
// RequireWallet falls back to either pending or its login-required state.
// We don't simulate Connected here — the panel's behaviour past that
// gate is what the role-dispatch test below covers in isolation.

#[test]
fn route_renders_without_panicking() {
    // Smoke test: contexts seeded, no panics, some HTML produced.
    let html = render_panel_disconnected();
    assert!(!html.is_empty());
}

#[test]
fn route_does_not_leak_role_panel_without_wallet() {
    // Without a WalletCtx, RequireWallet's "render" closure shouldn't
    // fire, so none of the TODO placeholders should appear.
    let html = render_panel_disconnected();
    assert!(!html.contains("Sub-component pending"),
        "leaked TODO placeholder without wallet:\n{html}");
}

use crate::components::coop_control::coop_control_panel::RoleSection;
use crate::components::tests_helper::fake_wallet;
use loan_machine_models::responses::{CooperativeView, ViewerRole};

fn fake_coop() -> CooperativeView {
    CooperativeView {
        id: "0xabc".into(),
        coop_id: "0xdeadbeef".into(),
        name: "Coop Teste".into(),
        loan_machine: "0x1111111111111111111111111111111111111111".into(),
        active: true,
        registered_at: 1_700_000_000,
    }
}

fn render_role(role: ViewerRole) -> String {
    render_to_string(move || view! {
        <RoleSection
            role=role
            wallet=fake_wallet()
            coop=fake_coop()
            on_state_changed=Callback::new(|_| {})
        />
    })
}

#[test]
fn visitor_role_shows_request_approval_placeholder() {
    assert!(render_role(ViewerRole::Visitor).contains("REQUEST APPROVAL FORM"));
}

#[test]
fn approval_pending_role_shows_pending_placeholder() {
    assert!(render_role(ViewerRole::ApprovalPending).contains("APPROVAL PENDING NOTICE"));
}

#[test]
fn approved_role_shows_first_vinculation_placeholder() {
    assert!(render_role(ViewerRole::Approved).contains("FIRST VINCULATION SECTION"));
}

#[test]
fn member_role_shows_member_placeholder() {
    assert!(render_role(ViewerRole::Member).contains("MEMBER PANEL"));
}

#[test]
fn moderator_role_shows_moderator_placeholder() {
    assert!(render_role(ViewerRole::Moderator).contains("MODERATOR PANEL"));
}

#[test]
fn admin_role_shows_admin_placeholder() {
    assert!(render_role(ViewerRole::Admin).contains("ADMIN PANEL"));
}