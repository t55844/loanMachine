// src/components/auth_bar_test.rs

use leptos::prelude::*;
use std::sync::Arc;
use crate::components::auth_bar::AuthBar;

fn render_auth_bar(wallet_state: Option<String>) -> String {
    let owner = Owner::new();
    owner.with(|| {
        let (wallet, _) = signal(wallet_state);
        view! {
            <AuthBar
                wallet=wallet
                on_login=Arc::new(|| {})
                on_logout=Arc::new(|| {})
            />
        }.to_html()
    })
}

// ── DISCONNECTED STATE ────────────────────────────────────────────────────

#[test]
fn disconnected_has_correct_class() {
    assert!(render_auth_bar(None).contains(r#"class="auth-bar auth-bar-disconnected""#));
}

#[test]
fn disconnected_shows_privy_icon() {
    assert!(render_auth_bar(None).contains("◈"));
}

#[test]
fn disconnected_shows_wallet_label() {
    assert!(render_auth_bar(None).contains("CARTEIRA DIGITAL"));
}

#[test]
fn disconnected_shows_privy_description() {
    let html = render_auth_bar(None);
    assert!(html.contains("Privy cria uma carteira blockchain para você via e-mail ou Google."));
    assert!(html.contains("Sem extensão, sem seed phrase."));
    assert!(html.contains("Sua chave fica protegida no enclave seguro deles."));
}

#[test]
fn disconnected_shows_connect_button() {
    let html = render_auth_bar(None);
    assert!(html.contains(r#"class="auth-bar-btn auth-bar-btn-connect""#));
}

#[test]
fn disconnected_button_has_off_dot() {
    assert!(render_auth_bar(None).contains(r#"class="auth-bar-btn-dot auth-bar-btn-dot-off""#));
}

#[test]
fn disconnected_does_not_show_connected_elements() {
    let html = render_auth_bar(None);
    assert!(!html.contains("CARTEIRA CONECTADA"));
    assert!(!html.contains("DESCONECTAR"));
    assert!(!html.contains("auth-bar-connected"));
}

// ── CONNECTED STATE ───────────────────────────────────────────────────────

#[test]
fn connected_has_correct_class() {
    let html = render_auth_bar(Some("0x1234567890abcdef".to_string()));
    assert!(html.contains(r#"class="auth-bar auth-bar-connected""#));
}

#[test]
fn connected_shows_connected_label() {
    let html = render_auth_bar(Some("0x1234567890abcdef".to_string()));
    assert!(html.contains("CARTEIRA CONECTADA"));
}

#[test]
fn connected_shows_on_dot() {
    let html = render_auth_bar(Some("0x1234567890abcdef".to_string()));
    assert!(html.contains(r#"class="auth-bar-btn-dot auth-bar-btn-dot-on""#));
}

#[test]
fn connected_shows_logout_button() {
    let html = render_auth_bar(Some("0x1234567890abcdef".to_string()));
    assert!(html.contains("DESCONECTAR"));
    assert!(html.contains(r#"class="auth-bar-btn auth-bar-btn-logout""#));
}

#[test]
fn connected_does_not_show_disconnected_elements() {
    let html = render_auth_bar(Some("0x1234567890abcdef".to_string()));
    assert!(!html.contains(r#"class="auth-bar-btn auth-bar-btn-connect""#));
    assert!(!html.contains("auth-bar-disconnected"));
    assert!(!html.contains("Privy cria"));
}

// ── ADDRESS TRUNCATION ────────────────────────────────────────────────────

#[test]
fn long_address_is_truncated() {
    // addr.len() > 12 → format!("{}...{}", &addr[..6], &addr[len-4..])
    let html = render_auth_bar(Some("0x1234567890abcdef".to_string()));
    assert!(html.contains("0x1234...cdef"));
}

#[test]
fn short_address_is_not_truncated() {
    // addr.len() <= 12 → shown as-is
    let html = render_auth_bar(Some("0x1234".to_string()));
    assert!(html.contains("0x1234"));
    assert!(!html.contains("..."));
}

#[test]
fn address_at_exact_boundary_is_not_truncated() {
    // len == 12 → the condition is > 12, so this should NOT truncate
    let boundary = "123456789012"; // exactly 12 chars
    let html = render_auth_bar(Some(boundary.to_string()));
    assert!(html.contains(boundary));
    assert!(!html.contains("..."));
}