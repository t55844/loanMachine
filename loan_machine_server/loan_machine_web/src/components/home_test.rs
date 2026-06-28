// src/components/home_tests.rs
//
// Run with:  cargo test --features ssr

use leptos::prelude::*;
use crate::components::home::{
    HeroSection, FeaturesSection, CtaSection, HomeFooter,
};

fn render_hero() -> String {
    let owner = Owner::new();
    owner.with(|| {
        view! { <HeroSection on_login=Callback::new(|_| {}) /> }.to_html()
    })
}

#[test]
fn hero_section_has_hero_class() {
    assert!(render_hero().contains(r#"class="hero""#));
}

#[test]
fn hero_headline_contains_keywords() {
    let html = render_hero();
    assert!(html.contains("COOPERATIVE"));
    assert!(html.contains("LENDING"));
    assert!(html.contains("ON CHAIN"));
    assert!(html.contains(r#"class="t-yellow""#));
}

#[test]
fn hero_section_description_paragraph() {
    let html = render_hero();
    assert!(html.contains("transparent, member-governed loan platform"));
    assert!(html.contains("no central custody"));
}

#[test]
fn hero_section_cta_buttons() {
    let html = render_hero();
    assert!(html.contains("JOIN THE COOPERATIVE"));
    assert!(html.contains(r#"class="btn btn-primary btn-lg""#));
    assert!(html.contains("SEE FEATURES"));
    assert!(html.contains(r##"href="#features""##));
}

fn render_features() -> String {
    let owner = Owner::new();
    owner.with(|| {
        view! { <FeaturesSection /> }.to_html()
    })
}

#[test]
fn features_section_anchor_and_title() {
    let html = render_features();
    assert!(html.contains(r#"id="features""#));
    assert!(html.contains("PLATFORM FEATURES"));
}

#[test]
fn features_carousel_has_six_slides() {
    let html = render_features();
    // 6 buttons + 1 container all reference "carousel-dot"
    assert!(html.matches("carousel-dot").count() >= 6);
    // Exactly one slide and one dot are active at SSR time (index 0)
    assert_eq!(html.matches("carousel-slide-active").count(), 1);
    assert_eq!(html.matches("carousel-dot-active").count(), 1);
}

#[test]
fn features_carousel_has_arrows_and_dots() {
    let html = render_features();
    assert!(html.contains("carousel-arrow-prev"));
    assert!(html.contains("carousel-arrow-next"));
    assert!(html.contains("carousel-dot"));
}

#[test]
fn features_slide_order_coop_dashboard_first() {
    // In the reversed order, Coop Dashboard is slide 0 (first in DOM)
    let html = render_features();
    let coop_pos = html.find("COOPERATIVE DASHBOARD").unwrap_or(usize::MAX);
    let donate_pos = html.find("DONATE").unwrap_or(usize::MAX);
    assert!(coop_pos < donate_pos, "Coop Dashboard must appear before Donate in the DOM");
}

#[test]
fn features_slide_order_donate_last() {
    let html = render_features();
    let request_pos = html.find("REQUEST A LOAN").unwrap_or(usize::MAX);
    let open_pos    = html.find("OPEN MARKET").unwrap_or(usize::MAX);
    let donate_pos  = html.find("DONATE &amp; WITHDRAW").unwrap_or(usize::MAX); // HTML-escaped
    assert!(request_pos < donate_pos);
    assert!(open_pos    < donate_pos);
}

#[test]
fn features_all_six_present() {
    let html = render_features();
    assert!(html.contains("COOPERATIVE DASHBOARD"));
    assert!(html.contains("MY PAYMENTS"));
    assert!(html.contains("MY LOANS"));
    assert!(html.contains("REQUEST A LOAN"));
    assert!(html.contains("OPEN MARKET"));
    assert!(html.contains("DONATE &amp; WITHDRAW")); // & is HTML-escaped from "DONATE & WITHDRAW"
}

#[test]
fn features_coop_dashboard_mock_content() {
    let html = render_features();
    assert!(html.contains("MONEY FLOW"));
    assert!(html.contains("TOTAL DONATIONS"));
    assert!(html.contains("OUTSTANDING LOANS"));
    assert!(html.contains("1119300 USDT"));
}

#[test]
fn features_my_payments_mock_content() {
    let html = render_features();
    assert!(html.contains("Repayment progress"));
    assert!(html.contains("33.333333 USDT"));
    assert!(html.contains("PAY"));
}

#[test]
fn features_my_loans_mock_content() {
    let html = render_features();
    assert!(html.contains("MY LOAN REQUISITIONS"));
    assert!(html.contains("CANCEL"));
    assert!(html.contains("mock-tab-active"));
}

#[test]
fn features_request_loan_mock_content() {
    let html = render_features();
    assert!(html.contains("AMOUNT (USDT)"));
    assert!(html.contains("NUMBER OF INSTALLMENTS"));
    assert!(html.contains("PAYMENT INTERVAL (DAYS)"));
}

#[test]
fn features_open_market_mock_content() {
    let html = render_features();
    assert!(html.contains("Your contribution"));
    assert!(html.contains("COVER 13%"));
}

#[test]
fn features_donate_mock_content() {
    let html = render_features();
    assert!(html.contains("Two signatures are required"));
    assert!(html.contains("WITHDRAW"));
}

fn render_cta() -> String {
    let owner = Owner::new();
    owner.with(|| {
        view! { <CtaSection on_login=Callback::new(|_| {}) /> }.to_html()
    })
}

#[test]
fn cta_section() {
    let html = render_cta();
    assert!(html.contains("READY TO"));
    assert!(html.contains("JOIN IN?"));
    assert!(html.contains("You will need your cooperative's access code to link up."));
    assert!(html.contains("GET STARTED"));
    assert!(html.contains(r#"class="btn btn-primary btn-lg""#));
}

fn render_footer() -> String {
    view! { <HomeFooter /> }.to_html()
}

#[test]
fn footer_section() {
    assert!(render_footer().contains("LOAN MACHINE"));
    assert!(render_footer().contains("Immutable contracts. Real community."));
}
