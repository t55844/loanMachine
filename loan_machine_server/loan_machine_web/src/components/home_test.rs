// src/components/home_tests.rs
//
// Run with:  cargo test --features ssr

use leptos::prelude::*;
use crate::components::home::{ HeroSection, HowItWorksSection, TransparencySection, CtaSection, HomeFooter};


fn render_hero() -> String {
    let owner = Owner::new();
    owner.with(|| {
        view! { <HeroSection on_login=Callback::new(|_| {}) /> }.to_html()
    })
}

#[test]
fn hero_section_headline() {

    assert!(render_hero().contains(r#"class="hero""#));
    assert!(render_hero().contains("Active Community"));
    assert!(render_hero().contains("CREDIT"));
    assert!(render_hero().contains("SOLIDARITY"));
    assert!(render_hero().contains("ON THE BLOCKCHAIN"));
    assert!(render_hero().contains(r#"class="t-yellow""#));
    assert!(render_hero().contains("SOLIDARITY"));
}

#[test]
fn hero_section_description_paragraph() {
    let html = render_hero();
    assert!(html.contains("A collective and transparent lending machine."));
    assert!(html.contains("Every transaction recorded immutably."));
    assert!(html.contains("Governed by its own members."));
}

#[test]
fn hero_section_cta_buttons() {
     let html = render_hero();
     assert!(html.contains("JOIN THE COOPERATIVE"));
     assert!(html.contains(r#"class="btn btn-primary btn-lg""#));
}

#[test]
fn hero_section_ghost_link() {
     let html = render_hero();
     assert!(html.contains("HOW IT WORKS"));
     assert!(html.contains(r##"href="#how-it-works""##));
     assert!(html.contains(r#"class="btn btn-ghost btn-lg""#));
}

#[test]
fn hero_section_stats_blocks() {
     let html = render_hero();
     assert!(html.contains("Active Members"));
     assert!(html.contains("0+"));
     assert!(html.contains("Total Donated"));
     assert!(html.contains("$ 0"));
     assert!(html.contains("Loans"));
     assert!(html.contains(">0<")); // bare zero, not "0+" or "$ 0"
}

fn render_how_it_works() -> String {
    view! { <HowItWorksSection /> }.to_html()
}

#[test]
fn how_it_works_section() {
    assert!(render_how_it_works().contains(r#"id="how-it-works""#));
    assert!(render_how_it_works().contains("HOW IT WORKS"));
    assert!(render_how_it_works().contains(">01<"));
    assert!(render_how_it_works().contains(">02<"));
    assert!(render_how_it_works().contains(">03<"));
}

#[test]
fn how_it_works_card_content() {
    // ── HOW IT WORKS: card titles ─────────────────────────────────────────
    assert!(render_how_it_works().contains("JOIN THE COOP"));
    assert!(render_how_it_works().contains("CONTRIBUTE"));
    assert!(render_how_it_works().contains("ACCESS CREDIT"));
    // ── HOW IT WORKS: card body copy ──────────────────────────────────────
    assert!(render_how_it_works().contains("Your digital wallet is created automatically"));
    assert!(render_how_it_works().contains("Make a donation in USDT to the collective fund."));
    assert!(render_how_it_works().contains("Request loans covered by your cooperative's members."));
}

fn render_transparency() -> String {
    view! { <TransparencySection /> }.to_html()
}

#[test]
fn transparency_contracts_on_chain() {
    let html = render_transparency();
    assert!(html.contains("On-Chain Contracts"));
    assert!(html.contains("100%"));
}

#[test]
fn trasparency_audit_traits(){
    let html = render_transparency();
    assert!(html.contains("Auditable Code"));
    assert!(html.contains("No Central Custody"));
    assert!(html.contains("Member Governance"));
    assert_eq!(render_transparency().matches("✓").count(), 3);
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
        assert!(render_cta().contains("JOIN IN?"));
        assert!(render_cta().contains("You will need your cooperative's access code to link up."));
        assert!(render_cta().contains("GET STARTED"));
        assert!( render_cta().contains(r#"class="btn btn-primary btn-lg""#));
}

fn render_footer() -> String {
    view! { <HomeFooter  /> }.to_html()
}
#[test]
fn footer_section() {
    assert!(render_footer().contains("LOAN MACHINE"));
    assert!(render_footer().contains("Immutable contracts. Real community."));
}
