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
    assert!(render_hero().contains("Comunidade Ativa"));
    assert!(render_hero().contains("CRÉDITO"));
    assert!(render_hero().contains("SOLIDÁRIO"));
    assert!(render_hero().contains("NA BLOCKCHAIN"));
    assert!(render_hero().contains(r#"class="t-yellow""#));
    assert!(render_hero().contains("SOLIDÁRIO"));
}

#[test]
fn hero_section_description_paragraph() {
    let html = render_hero();
    assert!(html.contains("Uma máquina de empréstimos coletiva e transparente."));
    assert!(html.contains("Cada transação registrada imutavelmente."));
    assert!(html.contains("Governada pelos próprios membros."));
}

#[test]
fn hero_section_cta_buttons() {
     let html = render_hero();
     assert!(html.contains("ENTRAR NA COOPERATIVA"));
     assert!(html.contains(r#"class="btn btn-primary btn-lg""#));
}

#[test]
fn hero_section_ghost_link() {
     let html = render_hero();
     assert!(html.contains("COMO FUNCIONA"));
     assert!(html.contains(r##"href="#como-funciona""##));
     assert!(html.contains(r#"class="btn btn-ghost btn-lg""#));
}

#[test]
fn hero_section_stats_blocks() {
     let html = render_hero();
     assert!(html.contains("Membros Ativos"));
     assert!(html.contains("0+"));
     assert!(html.contains("Total Doado"));
     assert!(html.contains("R$ 0"));
     assert!(html.contains("Empréstimos"));
     assert!(html.contains(">0<")); // bare zero, not "0+" or "R$ 0"
}

fn render_how_it_works() -> String {
    view! { <HowItWorksSection /> }.to_html()
}

#[test]
fn how_it_works_section() {
    assert!(render_how_it_works().contains(r#"id="como-funciona""#));
    assert!(render_how_it_works().contains("COMO FUNCIONA"));
    assert!(render_how_it_works().contains(">01<"));
    assert!(render_how_it_works().contains(">02<"));
    assert!(render_how_it_works().contains(">03<"));
}

#[test]
fn how_it_works_card_content() {
    // ── HOW IT WORKS: card titles ─────────────────────────────────────────
    assert!(render_how_it_works().contains("ENTRE NA COOP"));
    assert!(render_how_it_works().contains("CONTRIBUA"));
    assert!(render_how_it_works().contains("ACESSE CRÉDITO"));
    // ── HOW IT WORKS: card body copy ──────────────────────────────────────
    assert!(render_how_it_works().contains("Sua carteira digital é criada automaticamente"));
    assert!(render_how_it_works().contains("Faça uma doação em USDT para o fundo coletivo."));
    assert!(render_how_it_works().contains("Solicite empréstimos cobertos pelos membros da sua cooperativa."));
}
 
fn render_transparency() -> String {
    view! { <TransparencySection /> }.to_html()
}

#[test]
fn transparency_contracts_on_chain() {
    let html = render_transparency();
    assert!(html.contains("Contratos na Chain"));
    assert!(html.contains("100%"));
}

#[test]
fn trasparency_audit_traits(){
    let html = render_transparency();
    assert!(html.contains("Código Auditável"));
    assert!(html.contains("Sem custódia central"));
    assert!(html.contains("Governança dos membros"));
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
        assert!(html.contains("PRONTO PARA FAZER"));
        assert!(html.contains("PARTE?"));
        assert!(render_cta().contains("PARTE?"));
        assert!(render_cta().contains("Você precisará do código de acesso da sua cooperativa para se vincular."));
        assert!(render_cta().contains("COMEÇAR AGORA"));
        assert!( render_cta().contains(r#"class="btn btn-primary btn-lg""#));
}

fn render_footer() -> String {
    view! { <HomeFooter  /> }.to_html()
}
#[test]
fn footer_section() {
    assert!(render_footer().contains("LOAN MACHINE"));
    assert!(render_footer().contains("Contratos imutáveis. Comunidade real."));
}

