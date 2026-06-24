// SSR snapshot tests for the PendingRequisitions (open market) component,
// plus unit tests for the pure-logic helpers that drive its display logic.
//
// LocalResource does not resolve during SSR, so Suspense always renders its
// fallback. Tests below cover:
//   • the static shell (card tag, loading spinner)
//   • absence of item-specific content in loading state
//   • max_affordable_pct — the most business-critical helper
//   • open_status_display / coverage_color — badge and bar colour logic
//   • fmt_borrower — address abbreviation

use leptos::prelude::*;
use crate::components::loan_requisition::pending_requisitions::{
    PendingRequisitions, max_affordable_pct, open_status_display, coverage_color, fmt_borrower,
};
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
        view! { <PendingRequisitions coop_id="0xabc" /> }
    })
}

// ── SSR shell structure ───────────────────────────────────────

#[test]
fn panel_shows_open_market_tag() {
    assert!(
        panel_html().contains("OPEN MARKET"),
        "expected card tag 'OPEN MARKET'",
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
fn panel_does_not_show_item_data_initially() {
    let html = panel_html();
    assert!(!html.contains("USDT"),    "amount should not appear before data loads");
    assert!(!html.contains("parcels"), "parcel count should not appear before data loads");
    assert!(!html.contains("REQ #"),   "req id should not appear before data loads");
}

#[test]
fn panel_does_not_show_cover_button_initially() {
    assert!(
        !panel_html().contains("COVER"),
        "cover button should not appear in SSR — list is pending",
    );
}

#[test]
fn panel_does_not_show_slider_initially() {
    let html = panel_html();
    assert!(
        !html.contains("type=\"range\""),
        "range slider should not appear in SSR — list is pending",
    );
}

// ── max_affordable_pct ────────────────────────────────────────

#[test]
fn max_pct_zero_when_withdrawable_is_zero() {
    assert_eq!(max_affordable_pct("0", "10000000", 100), 0);
}

#[test]
fn max_pct_zero_when_withdrawable_is_empty_string() {
    assert_eq!(max_affordable_pct("", "10000000", 100), 0);
}

#[test]
fn max_pct_zero_when_amount_is_zero() {
    assert_eq!(max_affordable_pct("10000000", "0", 100), 0);
}

#[test]
fn max_pct_zero_when_amount_is_non_numeric() {
    assert_eq!(max_affordable_pct("10000000", "not-a-number", 100), 0);
}

#[test]
fn max_pct_50_when_withdrawable_is_half_of_amount() {
    // 5 USDT of 10 USDT → can cover 50%
    assert_eq!(max_affordable_pct("5000000", "10000000", 100), 50);
}

#[test]
fn max_pct_100_when_withdrawable_covers_full_amount() {
    // 10 USDT of 10 USDT → 100%, remaining is also 100
    assert_eq!(max_affordable_pct("10000000", "10000000", 100), 100);
}

#[test]
fn max_pct_capped_at_remaining_when_balance_exceeds_amount() {
    // 20 USDT of 10 USDT → would be 200%, but only 30% remains → cap at 30
    assert_eq!(max_affordable_pct("20000000", "10000000", 30), 30);
}

#[test]
fn max_pct_capped_at_remaining_when_exact_fit() {
    // 5 USDT of 10 USDT → 50%, remaining is 40% → cap at 40
    assert_eq!(max_affordable_pct("5000000", "10000000", 40), 40);
}

#[test]
fn max_pct_partial_when_only_partially_affordable() {
    // 3 USDT of 10 USDT → floor(30%) of remaining 70%
    assert_eq!(max_affordable_pct("3000000", "10000000", 70), 30);
}

#[test]
fn max_pct_zero_when_remaining_is_zero() {
    // loan is already fully covered → no room left
    assert_eq!(max_affordable_pct("10000000", "10000000", 0), 0);
}

// ── open_status_display ───────────────────────────────────────

#[test]
fn zero_coverage_shows_pending() {
    let (label, _) = open_status_display(0);
    assert_eq!(label, "PENDING");
}

#[test]
fn nonzero_coverage_shows_partial() {
    let (label, _) = open_status_display(1);
    assert_eq!(label, "PARTIAL");

    let (label99, _) = open_status_display(99);
    assert_eq!(label99, "PARTIAL");
}

#[test]
fn zero_coverage_badge_is_yellow() {
    let (_, color) = open_status_display(0);
    assert!(color.contains("yellow"), "expected yellow badge for pending, got {color}");
}

#[test]
fn nonzero_coverage_badge_is_gold() {
    let (_, color) = open_status_display(50);
    assert!(color.contains("gold"), "expected gold badge for partial, got {color}");
}

// ── coverage_color ────────────────────────────────────────────

#[test]
fn zero_coverage_bar_is_gray() {
    assert!(coverage_color(0).contains("gray"), "expected gray for 0% coverage");
}

#[test]
fn partial_coverage_bar_is_gold() {
    assert!(coverage_color(1).contains("gold"),  "expected gold for 1% coverage");
    assert!(coverage_color(99).contains("gold"), "expected gold for 99% coverage");
}

#[test]
fn full_coverage_bar_is_green() {
    assert!(coverage_color(100).contains("green"), "expected green for 100% coverage");
}

// ── fmt_borrower ──────────────────────────────────────────────

#[test]
fn fmt_borrower_truncates_ethereum_address() {
    let addr = "0x1234567890abcdef1234567890abcdef12345678";
    let result = fmt_borrower(addr);
    assert!(result.contains("0x123456"), "should start with first 8 chars");
    assert!(result.contains("5678"),     "should end with last 4 chars");
    assert!(result.contains('…'),        "should contain ellipsis");
    assert!(result.len() < addr.len(),   "result should be shorter than original");
}

#[test]
fn fmt_borrower_keeps_short_address_unchanged() {
    // address shorter than 12 chars → no truncation
    let addr = "0xabc";
    assert_eq!(fmt_borrower(addr), "0xabc");
}

#[test]
fn fmt_borrower_truncates_any_address_12_chars_or_longer() {
    // exactly 12 characters → truncation kicks in
    let addr = "0x1234567890";  // length 12
    let result = fmt_borrower(addr);
    assert_eq!(&result[..8], "0x123456");
    assert_eq!(&result[result.len()-4..], "7890");
}
