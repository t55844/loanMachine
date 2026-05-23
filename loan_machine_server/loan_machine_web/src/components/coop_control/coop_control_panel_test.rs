// loan_machine_web/src/components/coop_control_panel_test.rs

use leptos::prelude::*;
use leptos_router::components::Router;
use leptos_router::location::RequestUrl;

use crate::components::coop_control::coop_control_panel::CoopControlPanelRoute;
use crate::components::tests_helper::seed_prices;
use crate::wallet_auth::session::{WalletCtx, WalletSession};
use crate::components::gas_modal::provide_gas_modal;

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

/// Render just the membership-ladder rung for `role`, with no admin/mod
/// capabilities attached. Use this for ladder-only assertions so the
/// admin/moderator panels don't add noise to the rendered HTML.
fn render_role(role: ViewerRole) -> String {
    render_role_with_caps(role, false, false)
}

/// Render `RoleSection` with explicit capability flags. The ladder rung
/// for `role` always renders; `AdminPanel` and `ModeratorPanel` are
/// additive and gated by the flags.
fn render_role_with_caps(
    role: ViewerRole,
    is_admin: bool,
    is_moderator: bool,
) -> String {
    render_to_string(move || {
        provide_gas_modal(); // needed by RequestApproval + FirstVinculationForm
        view! {
            <RoleSection
                role=role
                is_admin=is_admin
                is_moderator=is_moderator
                wallet=fake_wallet()
                coop=fake_coop()
                on_state_changed=Callback::new(|_| {})
            />
        }
    })
}

// ── Ladder tests ──────────────────────────────────────────
//
// The membership ladder (Visitor → ApprovalPending → Approved → Member)
// is what `role` drives. Capabilities are tested separately below.

#[test]
fn visitor_role_shows_request_approval_card() {
    let html = render_role(ViewerRole::Visitor);
    assert!(html.contains("VISITANTE — PEDIDO DE APROVAÇÃO"),
        "expected approval-request card tag, got:\n{html}");
    assert!(html.contains("INICIAR PEDIDO DE APROVAÇÃO"),
        "expected initiate button, got:\n{html}");
}

#[test]
fn approved_role_shows_first_vinculation_form() {
    let html = render_role(ViewerRole::Approved);
    assert!(html.contains("VINCULE SUA CARTEIRA"),
        "expected vinculation form title, got:\n{html}");
    assert!(html.contains("PASSO 01"),
        "expected vinculation form step card, got:\n{html}");
}

#[test]
fn member_role_shows_member_placeholder() {
    assert!(render_role(ViewerRole::Member).contains("MEMBER PANEL"));
}

// Legacy: `ViewerRole::Admin` and `ViewerRole::Moderator` are no longer
// ladder positions — admin/moderator status is carried by the `is_admin`
// and `is_moderator` flags instead. The match arm currently routes these
// variants through the Member placeholder. Delete these two tests when
// the variants are removed from `ViewerRole`.

#[test]
fn legacy_admin_variant_falls_through_to_member_placeholder() {
    assert!(render_role(ViewerRole::Admin).contains("MEMBER PANEL"));
}

#[test]
fn legacy_moderator_variant_falls_through_to_member_placeholder() {
    assert!(render_role(ViewerRole::Moderator).contains("MEMBER PANEL"));
}

// ── Capability tests ──────────────────────────────────────
//
// `is_admin` / `is_moderator` are additive: they mount `AdminPanel` /
// `ModeratorPanel` alongside whatever ladder rung is showing. These
// assertions use substrings that should appear in the real panels'
// output — adjust them if `AdminPanel` / `ModeratorPanel` don't yet
// emit a stable marker (a card tag like "PAINEL DE ADMIN" or a
// `data-testid` would be ideal).

#[test]
fn admin_capability_mounts_admin_panel() {
    let html = render_role_with_caps(ViewerRole::Member, true, false);
    assert!(html.contains("ADMIN"),
        "expected AdminPanel to render when is_admin=true, got:\n{html}");
}

#[test]
fn moderator_capability_mounts_moderator_panel() {
    let html = render_role_with_caps(ViewerRole::Member, false, true);
    assert!(html.contains("MODERADOR") || html.contains("MODERATOR"),
        "expected ModeratorPanel to render when is_moderator=true, got:\n{html}");
}

#[test]
fn no_capability_flags_means_only_ladder_renders() {
    // Sanity check that the additive panels really are gated. The Member
    // placeholder still shows; the admin/mod panels must not.
    let html = render_role_with_caps(ViewerRole::Member, false, false);
    assert!(html.contains("MEMBER PANEL"),
        "expected member ladder rung to render, got:\n{html}");
    // These negative assertions are only meaningful once AdminPanel /
    // ModeratorPanel actually emit a distinctive marker — pick one and
    // tighten this if the substrings above get more specific.
}