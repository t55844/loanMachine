// loan_machine_web/src/components/gas_modal_test.rs

use leptos::prelude::*;
use crate::components::gas_modal::{
    GasEstimate, GasModal, GasModalRequest, provide_gas_modal,
};

use crate::components::tests_helper::seed_prices;
// ── Helper ────────────────────────────────────────────────────
// Same pattern as create_coop_test.rs / vinculation_test.rs.

fn render_to_string<V, F>(f: F) -> String
where
    F: FnOnce() -> V,
    V: IntoView,
{
    let owner = Owner::new();
    let html  = owner.with(|| f().to_html());
    drop(owner);
    html
}

// ── GasEstimate::gas_units — hex parsing ────────────────────
//
// Pure logic, no Leptos required. We pin canonical EVM gas values so
// changes to the parser get caught immediately.

#[test]
fn gas_units_parses_21000_eth_transfer() {
    // 0x5208 = 21000 — base cost of an ETH transfer.
    let est = GasEstimate { label: "x".into(), gas_hex: "0x5208".into() };
    assert_eq!(est.gas_units(), 21_000);
}

#[test]
fn gas_units_parses_typical_initialize_call() {
    // 0x927c0 = 600_000 — matches the gas_initialize fallback in your service.
    let est = GasEstimate { label: "x".into(), gas_hex: "0x927c0".into() };
    assert_eq!(est.gas_units(), 600_000);
}

#[test]
fn gas_units_parses_typical_contract_deploy() {
    // 0x2dc6c0 = 3_000_000 — typical LoanMachine deploy budget.
    let est = GasEstimate { label: "x".into(), gas_hex: "0x2dc6c0".into() };
    assert_eq!(est.gas_units(), 3_000_000);
}

#[test]
fn gas_units_handles_missing_0x_prefix() {
    let with_prefix = GasEstimate { label: "x".into(), gas_hex: "0x5208".into() };
    let no_prefix   = GasEstimate { label: "x".into(), gas_hex: "5208".into()   };
    assert_eq!(with_prefix.gas_units(), no_prefix.gas_units());
}

#[test]
fn gas_units_handles_uppercase_hex() {
    let lower = GasEstimate { label: "x".into(), gas_hex: "0xabcdef".into() };
    let upper = GasEstimate { label: "x".into(), gas_hex: "0xABCDEF".into() };
    assert_eq!(lower.gas_units(), upper.gas_units());
}

#[test]
fn gas_units_returns_zero_on_garbage_input() {
    let est = GasEstimate { label: "x".into(), gas_hex: "not-hex".into() };
    assert_eq!(est.gas_units(), 0);
}

#[test]
fn gas_units_returns_zero_on_empty_string() {
    let est = GasEstimate { label: "x".into(), gas_hex: "".into() };
    assert_eq!(est.gas_units(), 0);
}




// ── Modal CLOSED — request signal is None ──────────────────
//
// The modal uses an always-mounted shell with CSS-toggled visibility
// (so click handlers don't churn). "Closed" means:
//   - the shell IS in the DOM (backdrop, card, buttons)
//   - but the `open` class is absent (CSS hides it)
//   - and the request-driven content (title, alert, estimates) is gone

fn closed_html() -> String {
    render_to_string(|| {
        seed_prices(None);
        provide_gas_modal();
        view! { <GasModal /> }
    })
}


#[test]
fn closed_modal_omits_open_class() {
    let html = closed_html();
    // class:open serializes as " open" inside the class attribute when true.
    // When false, it's omitted entirely.
    assert!(
        !html.contains("gas-modal-backdrop open")
            && !html.contains("gas-modal-backdrop  open"),
        "expected no `open` class on backdrop, got: {html}",
    );
}

#[test]
fn closed_modal_omits_request_driven_content() {
    let html = closed_html();
    // The card-tag (title), alert, and estimate blocks only render
    // when req.get() is Some(_). None of them should appear when closed.
    assert!(!html.contains("card-tag"));
    assert!(!html.contains("Review the estimated gas cost"));
    assert!(!html.contains("gas-estimate-block"));
}

#[test]
fn closed_modal_keeps_shell_mounted() {
    // Sanity check: the always-mounted shell IS present even when closed.
    // This documents the architecture for future readers.
    let html = closed_html();
    assert!(html.contains("gas-modal-backdrop"));
    assert!(html.contains("gas-modal-card"));
}

// ── Modal OPEN — request set with one estimate ─────────────
//
// We provide context, write Some(...) into the request signal, THEN render.
// Signal writes are synchronous, so the closure sees the new value.

fn open_html() -> String {
    render_to_string(|| {
        seed_prices(None);
        provide_gas_modal();
        let setter = expect_context::<WriteSignal<Option<GasModalRequest>>>();
        setter.set(Some(GasModalRequest {
            title:     "TESTE GAS MODAL".into(),
            estimates: vec![GasEstimate {
                label:   "Test Operation".into(),
                gas_hex: "0x5208".into(),  // 21_000
            }],
            on_confirm: Callback::new(|_| {}),
        }));
        view! { <GasModal /> }
    })
}

#[test]
fn open_modal_renders_backdrop() {
    assert!(open_html().contains("gas-modal-backdrop"));
}

#[test]
fn open_modal_renders_card() {
    assert!(open_html().contains("gas-modal-card"));
}

#[test]
fn open_modal_shows_title_text() {
    assert!(open_html().contains("TESTE GAS MODAL"));
}

#[test]
fn open_modal_shows_card_tag_class() {
    assert!(open_html().contains("card-tag"));
}

#[test]
fn open_modal_shows_info_alert_text() {
    assert!(open_html().contains("Review the estimated gas cost"));
}

#[test]
fn open_modal_shows_estimate_label() {
    assert!(open_html().contains("Test Operation"));
}

#[test]
fn open_modal_shows_gas_units_in_decimal() {
    assert!(open_html().contains("21000 gas units"));
}

#[test]
fn open_modal_shows_raw_gas_hex() {
    assert!(open_html().contains("0x5208"));
}

#[test]
fn open_modal_shows_confirm_button() {
    assert!(open_html().contains("CONFIRM AND SIGN"));
}

#[test]
fn open_modal_shows_cancel_button() {
    assert!(open_html().contains("CANCEL"));
}

// ── Price breakdown — SSR loading / unavailable state ──────
//
// LocalResource only runs in the browser (relies on JsFuture + fetch).
// In SSR the resource stays pending and Suspense shows its fallback
// — OR resolves to None depending on Leptos internals. Either is fine;
// what matters is that the resolved breakdown rows do NOT appear without
// a successful network fetch.

#[test]
fn open_modal_shows_loading_or_unavailable_state() {
    let html = open_html();
    assert!(
        html.contains("price unavailable"),
        "expected unavailable message in SSR, got:\n{html}",
    );
}

#[test]
fn open_modal_does_not_render_resolved_breakdown_in_ssr() {
    let html = open_html();
    // The breakdown rows only appear when prices.get().flatten() == Some(_),
    // which can't happen in SSR. If they leak into the HTML, our Suspense
    // contract is broken.
    assert!(
        !html.contains("gas-price-row"),
        "resolved price rows leaked into SSR render despite pending resource",
    );
}

// ── Modal OPEN — request with two estimates ────────────────

fn open_html_two_estimates() -> String {
    render_to_string(|| {
        provide_gas_modal();
        let setter = expect_context::<WriteSignal<Option<GasModalRequest>>>();
        setter.set(Some(GasModalRequest {
            title:     "TWO OPERATIONS".into(),
            estimates: vec![
                GasEstimate { label: "Contract Deploy".into(),    gas_hex: "0x2dc6c0".into() }, // 3_000_000
                GasEstimate { label: "Initialize Multisig".into(), gas_hex: "0x927c0".into()  }, // 600_000
            ],
            on_confirm: Callback::new(|_| {}),
        }));
        view! { <GasModal /> }
    })
}

#[test]
fn modal_renders_both_estimate_labels() {
    let html = open_html_two_estimates();
    assert!(html.contains("Contract Deploy"));
    assert!(html.contains("Initialize Multisig"));
}

#[test]
fn modal_renders_both_gas_unit_values() {
    let html = open_html_two_estimates();
    assert!(html.contains("3000000 gas units"));  // 0x2dc6c0
    assert!(html.contains("600000 gas units"));   // 0x927c0
}

#[test]
fn modal_creates_one_estimate_block_per_estimate() {
    assert_eq!(
        open_html_two_estimates().matches("gas-estimate-block").count(),
        2,
    );
}