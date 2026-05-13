// src/components/auth_bar_test.rs

use leptos::prelude::*;
use loan_machine_models::wallet_address::WalletAddress;

use crate::components::auth_bar::AuthBar;
use crate::components::tests_helper::fake_wallet;
use crate::wallet_auth::session::{WalletCtx, WalletSession};

// ── Render helpers ────────────────────────────────────────────────────────
//
// AuthBar reads WalletCtx via expect_context, so every test needs to seed
// the context before rendering. One helper per state.

fn render_with(session: WalletSession) -> String {
    let owner = Owner::new();
    owner.with(|| {
        let (s, set) = signal(session);
        provide_context(WalletCtx { session: s, set });
        view! { <AuthBar/> }.to_html()
    })
}

fn render_disconnected() -> String {
    render_with(WalletSession::Disconnected)
}

fn render_connected(wallet: WalletAddress) -> String {
    render_with(WalletSession::Connected { wallet })
}

fn render_restoring() -> String {
    render_with(WalletSession::Restoring)
}



// ── DISCONNECTED ──────────────────────────────────────────────────────────



#[test]
fn disconnected_shows_connect_button() {
    assert!(render_disconnected().contains("CONECTAR"));
}

#[test]
fn disconnected_hides_connected_elements() {
    let html = render_disconnected();
    assert!(!html.contains("DESCONECTAR"));
    assert!(!html.contains("auth-bar-btn-dot-on"));
}

// ── CONNECTED ─────────────────────────────────────────────────────────────

#[test]
fn connected_shows_short_wallet_address() {
    // The component renders wallet.short() — verify the truncated form is
    // in the output. Computing it from the wallet keeps the test honest:
    // if short() changes shape, this test still passes for the right reason.
    let wallet = fake_wallet();
    let html = render_connected(wallet);
    assert!(html.contains(&wallet.short()));
}

#[test]
fn connected_shows_logout_button() {
    assert!(render_connected(fake_wallet()).contains("DESCONECTAR"));
}

#[test]
fn connected_shows_on_dot() {
    assert!(render_connected(fake_wallet())
        .contains(r#"class="auth-bar-btn-dot auth-bar-btn-dot-on""#));
}

#[test]
fn connected_hides_disconnected_elements() {
    let html = render_connected(fake_wallet());
    assert!(!html.contains("CARTEIRA NÃO CONECTADA"));
    assert!(!html.contains(">CONECTAR<"));  // anchor on tag boundary so it
                                            // doesn't match "DESCONECTAR"
}

// ── RESTORING ─────────────────────────────────────────────────────────────

#[test]
fn restoring_shows_label() {
    assert!(render_restoring().contains("RESTAURANDO"));
}

#[test]
fn restoring_shows_pending_dot() {
    assert!(render_restoring()
        .contains(r#"class="auth-bar-btn-dot auth-bar-btn-dot-pending""#));
}

#[test]
fn restoring_shows_no_buttons() {
    // No CONECTAR / DESCONECTAR during restore — the UI is read-only.
    let html = render_restoring();
    assert!(!html.contains("CONECTAR"));
    assert!(!html.contains("DESCONECTAR"));
}