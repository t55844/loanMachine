//! Wire-format regression tests for bundles consumed by privy-bridge.js.
//!
//! WHY THESE EXIST
//! ───────────────
//! The JS bridge reads bundle fields by exact name. A Rust-side rename
//! (e.g. join_calldata → joinCalldata) compiles fine but produces a tx
//! with undefined `to`/`data`/`gas`, which Privy signs as zeros, which
//! Anvil rejects with "intrinsic gas too low". This is a hard bug to
//! diagnose because the failure surfaces three layers down. Lock the
//! shape here so renames break loudly.
//!
//! If you change one of these field names, also update privy-bridge.js
//! and the Rust→JS mapping functions (send_bundle_to_privy, js_send_tx).

use loan_machine_models::responses::{CoopDeployBundle, VinculationBundle};
use serde_json::Value;

fn parse_hex_u64(s: &str) -> Option<u64> {
    u64::from_str_radix(s.strip_prefix("0x")?, 16).ok()
}

// ── VinculationBundle ────────────────────────────────────────

fn sample_vinc_bundle() -> VinculationBundle {
    VinculationBundle {
        join_calldata:        "0xdeadbeef".into(),
        loan_machine_address: "0x0000000000000000000000000000000000000001".into(),
        factory_address:      "0x0000000000000000000000000000000000000002".into(),
        gas_join:             "0x2eacc".into(),
    }
}

#[test]
fn vinc_bundle_serializes_with_keys_bridge_reads() {
    let v: Value = serde_json::to_value(sample_vinc_bundle()).unwrap();
    let obj = v.as_object().expect("must be a JSON object");

    // These exact keys are read by send_bundle_to_privy in vinculation.rs
    // when constructing the {to, data, gas} payload for the JS bridge.
    for required in ["join_calldata", "loan_machine_address", "gas_join"] {
        assert!(obj.contains_key(required),
            "VinculationBundle missing key `{required}` — JS bridge will see undefined. \
             Got keys: {:?}", obj.keys().collect::<Vec<_>>());
    }
}

#[test]
fn vinc_bundle_gas_is_hex_prefixed() {
    let b = sample_vinc_bundle();
    assert!(b.gas_join.starts_with("0x"),
        "gas_join must be 0x-prefixed hex (consumed by eth_sendTransaction); \
         got {:?}", b.gas_join);
    assert!(parse_hex_u64(&b.gas_join).is_some(),
        "gas_join must parse as hex u64; got {:?}", b.gas_join);
}

#[test]
fn vinc_bundle_addresses_are_hex_prefixed() {
    let b = sample_vinc_bundle();
    assert!(b.loan_machine_address.starts_with("0x"));
    assert!(b.factory_address.starts_with("0x"));
    assert_eq!(b.loan_machine_address.len(), 42, "address must be 0x + 40 hex chars");
}

// ── CoopDeployBundle ─────────────────────────────────────────

fn sample_deploy_bundle() -> CoopDeployBundle {
    CoopDeployBundle {
        deploy_data:     "0x6080".into(),
        gas_deploy:      "0x51790e".into(),
        initialize_data: "0xabcd".into(),
        gas_initialize:  "0x927c0".into(),
        access_code:     "ABCDEF123456".into(),
    }
}

#[test]
fn deploy_bundle_serializes_with_keys_bridge_reads() {
    let v: Value = serde_json::to_value(sample_deploy_bundle()).unwrap();
    let obj = v.as_object().unwrap();

    for required in ["deploy_data", "gas_deploy", "initialize_data", "gas_initialize", "access_code"] {
        assert!(obj.contains_key(required),
            "CoopDeployBundle missing key `{required}`. Got: {:?}",
            obj.keys().collect::<Vec<_>>());
    }
}

#[test]
fn deploy_bundle_all_gas_fields_are_hex_prefixed() {
    let b = sample_deploy_bundle();
    for (name, value) in [("gas_deploy", &b.gas_deploy), ("gas_initialize", &b.gas_initialize)] {
        assert!(value.starts_with("0x"), "{name} must be 0x-hex; got {value:?}");
        assert!(parse_hex_u64(value).is_some(), "{name} must parse as hex u64");
    }
}

#[test]
fn deploy_bundle_access_code_is_human_friendly_length() {
    // The access code is shown to humans to share with cooperative members.
    // 12 chars from a 32-char alphabet is the chosen length; if it shrinks,
    // entropy drops; if it grows, UX suffers. Lock it down.
    let b = sample_deploy_bundle();
    assert_eq!(b.access_code.len(), 12);
}