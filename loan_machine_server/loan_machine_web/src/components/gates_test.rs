// src/components/gates_test.rs
use super::gates::{outcome, GateOutcome, LoginRequiredKind};
use crate::wallet_auth::{ WalletSession};
use loan_machine_models::wallet_address::WalletAddress;
use std::str::FromStr;

fn fake_addr() -> WalletAddress {
    WalletAddress::from_str(&"0x1234567890abcdef1234567890abcdef12345678")
        .expect("test fixture address must parse")
}

// ── outcome(): pure decision logic ────────────────────────────

#[test]
fn restoring_without_pending_yields_skeleton() {
    let session = WalletSession::Restoring;
    assert_eq!(outcome(&session, false), GateOutcome::Skeleton);
}

#[test]
fn restoring_with_pending_yields_pending() {
    let session = WalletSession::Restoring;
    assert_eq!(outcome(&session, true), GateOutcome::Pending);
}

#[test]
fn disconnected_yields_fallback_regardless_of_pending_prop() {
    // The "Disconnected" UX is the same whether the caller passed a
    // pending view or not — pending is only consulted while Restoring.
    let session = WalletSession::Disconnected;
    assert_eq!(outcome(&session, false), GateOutcome::Fallback);
    assert_eq!(outcome(&session, true),  GateOutcome::Fallback);
}

#[test]
fn connected_yields_render_with_wallet() {
    let addr = fake_addr();
    let session = WalletSession::Connected { wallet: addr.clone() };
    match outcome(&session, false) {
        GateOutcome::Render(got) => assert_eq!(got, &addr),
        other => panic!("expected Render(_), got {other:?}"),
    }
}

#[test]
fn connected_ignores_pending_prop() {
    // A connected user shouldn't see "loading" even if the caller
    // bothered to wire a pending view.
    let addr = fake_addr();
    let session = WalletSession::Connected { wallet: addr.clone() };
    match outcome(&session, true) {
        GateOutcome::Render(got) => assert_eq!(got, &addr),
        other => panic!("expected Render(_), got {other:?}"),
    }
}

// ── LoginRequiredKind: copy mapping ───────────────────────────

#[test]
fn login_required_kinds_have_distinct_copy() {
    let coop   = LoginRequiredKind::Cooperatives.body_text();
    let vinc   = LoginRequiredKind::Vinculation.body_text();
    let create = LoginRequiredKind::CreateCoop.body_text();
    assert_ne!(coop, vinc, "Cooperativas vs Vinculation copy collision");
    assert_ne!(coop, create, "Cooperativas vs CreateCoop copy collision");
    assert_ne!(vinc, create, "Vinculation vs CreateCoop copy collision");
}

#[test]
fn login_required_copy_mentions_wallet_or_connect() {
    // Sanity check: every variant's body mentions a domain term.
    // Cheap guard against accidental copy drift.
    for kind in [
        LoginRequiredKind::Cooperatives,
        LoginRequiredKind::Vinculation,
        LoginRequiredKind::CreateCoop,
    ] {
        let txt = kind.body_text();
        assert!(
            txt.contains("wallet") || txt.contains("CONNECT"),
            "copy missing domain term: {txt:?} (kind {kind:?})"
        );
    }
}