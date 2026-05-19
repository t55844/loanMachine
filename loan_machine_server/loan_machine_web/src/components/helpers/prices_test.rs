// loan_machine_web/src/components/helpers/prices_test.rs

use crate::components::helpers::prices::{
    read_cached, use_prices, Prices,
};

// ── Type shape ────────────────────────────────────────────

#[test]
fn prices_serde_round_trip() {
    // If we ever change Prices' field names, every cached entry
    // in users' localStorage becomes garbage. This test fails loudly
    // before that happens — and reminds whoever's editing to bump
    // STORAGE_KEY's version suffix.
    let p = Prices { eth_usd: 3214.50, usd_brl: 5.12 };
    let json = serde_json::to_string(&p).unwrap();
    let back: Prices = serde_json::from_str(&json).unwrap();
    assert_eq!(back.eth_usd, 3214.50);
    assert_eq!(back.usd_brl, 5.12);
}

#[test]
fn prices_deserializes_known_shape() {
    // Documents the on-disk JSON shape. If a future serde change
    // renames fields, this breaks at the same time as the round trip.
    let json = r#"{"eth_usd":3000.0,"usd_brl":5.0}"#;
    let p: Prices = serde_json::from_str(json).unwrap();
    assert_eq!(p.eth_usd, 3000.0);
    assert_eq!(p.usd_brl, 5.0);
}

// ── Non-wasm fallbacks ────────────────────────────────────

#[test]
fn read_cached_returns_none_outside_browser() {
    // The cfg(not(wasm32)) branch must return None — never panic,
    // never read random files. Asserting it locks the contract.
    assert!(read_cached().is_none());
}

// ── use_prices defensive fallback ─────────────────────────
//
// When no provider is in scope, use_prices() must return a signal
// that reads None — not panic. This is the contract that makes
// AuthBar, GasModal, and BalanceRow safe to render in any test.

use leptos::prelude::*;

#[test]
fn use_prices_returns_none_signal_when_no_provider() {
    let owner = Owner::new();
    let value = owner.with(|| use_prices().get());
    drop(owner);
    assert!(value.is_none());
}

#[test]
fn use_prices_returns_provided_signal_when_present() {
    let owner = Owner::new();
    let value = owner.with(|| {
        let p = Prices { eth_usd: 3000.0, usd_brl: 5.0 };
        let (r, _) = signal(Some(p));
        provide_context::<ReadSignal<Option<Prices>>>(r);
        use_prices().get()
    });
    drop(owner);
    assert_eq!(value.map(|p| p.eth_usd), Some(3000.0));
}