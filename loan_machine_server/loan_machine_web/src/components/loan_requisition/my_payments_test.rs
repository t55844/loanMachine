// SSR snapshot tests for the MyPayments component, plus unit tests for the
// pure helper functions that drive its display logic.
//
// LocalResource does not resolve during SSR, so Suspense always renders its
// fallback (the spinner). Tests cover:
//   • the static shell (card tag, loading spinner)
//   • absence of item-specific content before data loads
//   • fmt_usdt — USDT formatting helper
//   • fmt_date — Unix-timestamp-to-date-string helper

use leptos::prelude::*;
use crate::components::loan_requisition::my_payments::{MyPayments, fmt_usdt, fmt_date};
use crate::components::gas_modal::provide_gas_modal;

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

fn panel_html() -> String {
    render_to_string(|| {
        provide_gas_modal();
        view! { <MyPayments coop_id="0xabc" /> }
    })
}

// ── SSR shell structure ───────────────────────────────────────

#[test]
fn panel_shows_my_payments_tag() {
    assert!(
        panel_html().contains("MY PAYMENTS"),
        "expected card tag 'MY PAYMENTS'",
    );
}

#[test]
fn panel_shows_spinner_while_loading() {
    assert!(
        panel_html().contains("spinner"),
        "expected Suspense fallback to render a spinner",
    );
}

#[test]
fn panel_does_not_show_loan_data_initially() {
    let html = panel_html();
    assert!(!html.contains("REQ #"),       "req id should not appear before data loads");
    assert!(!html.contains("ACTIVE"),      "active badge should not appear before data loads");
    assert!(!html.contains("parcels paid"),"progress label should not appear before data loads");
    assert!(!html.contains("PAID"),        "parcel status badge should not appear before data loads");
    assert!(!html.contains("PENDING"),     "parcel pending badge should not appear before data loads");
}

#[test]
fn panel_does_not_show_pay_button_initially() {
    // The PAY button is inside Suspense and only rendered after the
    // LocalResource resolves.
    assert!(
        !panel_html().contains(">PAY<"),
        "pay button should not appear in SSR — list is pending",
    );
}

#[test]
fn panel_does_not_show_empty_state_initially() {
    // "No active loans" is rendered inside Suspense, so it only appears after
    // the resource resolves.
    assert!(
        !panel_html().contains("No active loans"),
        "empty-state message should not appear in SSR — list is pending",
    );
}

// ── fmt_usdt ─────────────────────────────────────────────────

#[test]
fn fmt_usdt_zero_renders_zero_usdt() {
    assert_eq!(fmt_usdt("0"), "0 USDT");
}

#[test]
fn fmt_usdt_whole_number_omits_decimal() {
    // 10_000_000 raw units = 10 USDT (6 decimals, zero remainder)
    assert_eq!(fmt_usdt("10000000"), "10 USDT");
}

#[test]
fn fmt_usdt_with_fractional_part_shows_six_decimal_places() {
    // 1_500_000 raw units = 1.500000 USDT
    assert_eq!(fmt_usdt("1500000"), "1.500000 USDT");
}

#[test]
fn fmt_usdt_one_micro_usdt() {
    // Smallest unit: 1 = 0.000001 USDT
    assert_eq!(fmt_usdt("1"), "0.000001 USDT");
}

#[test]
fn fmt_usdt_large_value_formats_correctly() {
    // 1_000_000_000_000 raw = 1_000_000 USDT (one million)
    assert_eq!(fmt_usdt("1000000000000"), "1000000 USDT");
}

#[test]
fn fmt_usdt_non_numeric_string_renders_zero() {
    assert_eq!(fmt_usdt("not-a-number"), "0 USDT");
}

#[test]
fn fmt_usdt_empty_string_renders_zero() {
    assert_eq!(fmt_usdt(""), "0 USDT");
}

// ── fmt_date ─────────────────────────────────────────────────

#[test]
fn fmt_date_zero_renders_dash() {
    assert_eq!(fmt_date(0), "—");
}

#[test]
fn fmt_date_one_day_after_epoch_is_1970_01_02() {
    // 86 400 s = exactly one day after the Unix epoch
    assert_eq!(fmt_date(86_400), "1970-01-02");
}

#[test]
fn fmt_date_known_timestamp_2024_01_01() {
    // 2024-01-01 00:00:00 UTC = 1 704 067 200 Unix seconds
    assert_eq!(fmt_date(1_704_067_200), "2024-01-01");
}

#[test]
fn fmt_date_leap_year_boundary_2000_02_29() {
    // 2000-02-29 00:00:00 UTC = 951 782 400 Unix seconds
    assert_eq!(fmt_date(951_782_400), "2000-02-29");
}

#[test]
fn fmt_date_end_of_year_2023_12_31() {
    // 2023-12-31 00:00:00 UTC = 1 703 980 800 Unix seconds
    assert_eq!(fmt_date(1_703_980_800), "2023-12-31");
}
