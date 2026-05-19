// loan_machine_web/src/components/create_coop_test.rs
//
// SSR snapshot tests for CreateCoopPage, CoopChoicePage, and StepProgress.
//
// WHAT THESE TESTS COVER
// ──────────────────────
// Initial SSR render only — the HTML produced before any signal update or
// user interaction. Reactive branches (AccessCode, SignInit, Done, etc.)
// are only reachable after signals change, which can't happen via spawn_local
// in SSR. Those branches need wasm-bindgen-test in a real browser.
//
// WHAT THEY DON'T COVER
// ──────────────────────
// - JS interop (js_deploy_contract, js_send_tx) — wasm32-only, never runs here.
// - The registration Effect — spawn_local futures aren't polled in SSR.
// - Step transitions triggered by privy_* CustomEvents.
//
// REFERENCE PATTERN
// ──────────────────
// Mirrors vinculation_test.rs: Owner scope, render_to_string helper, one
// assert per test so failures are individually pinpointed.

use leptos::prelude::*;
use crate::components::create_coop::create_coop::{CoopStep, CreateCoopPage};
use crate::components::create_coop::create_coop_steps::StepProgress;
use crate::components::create_coop::coop_choice::CoopChoicePage;
use crate::components::tests_helper::fake_wallet;
// ── Helper ────────────────────────────────────────────────────

/// Render any synchronous view inside a fresh Owner scope.
/// Owner is required because signals register with the current owner at creation.
fn render_to_string<V, F>(f: F) -> String
where
    F: FnOnce() -> V,
    V: IntoView,
{
    let owner = Owner::new();
    let html = owner.with(|| f().to_html());
    drop(owner);
    html
}

// ── CoopChoicePage ────────────────────────────────────────────

fn choice_html() -> String {
    render_to_string(|| view! { <CoopChoicePage /> })
}

#[test]
fn choice_shows_section_title() {
    assert!(choice_html().contains("O QUE DESEJA FAZER"));
}

#[test]
fn choice_shows_vinculate_card_text() {
    assert!(choice_html().contains("ENTRAR EM UMA"));
}

#[test]
fn choice_shows_create_coop_card_text() {
    assert!(choice_html().contains("CRIAR UMA NOVA"));
}

#[test]
fn choice_vinculate_link_is_correct() {
    assert!(choice_html().contains(r#"href="/vinculation""#));
}

#[test]
fn choice_create_coop_link_is_correct() {
    assert!(choice_html().contains(r#"href="/create-coop""#));
}

#[test]
fn choice_shows_vinculate_cta_button() {
    assert!(choice_html().contains("VINCULAR DOCUMENTO"));
}

#[test]
fn choice_shows_create_coop_cta_button() {
    assert!(choice_html().contains("CRIAR COOPERATIVA"));
}

// ── CreateCoopPage — Step 1 (Form) ───────────────────────────
//
// The page always mounts in CoopStep::Form — this is the SSR initial render.

fn page_html() -> String {
    render_to_string(|| view! {
        <CreateCoopPage founder_wallet=fake_wallet() />
    })
}

#[test]
fn page_shows_main_title() {
    assert!(page_html().contains("CRIAR COOPERATIVA"));
}

#[test]
fn page_shows_step_progress_first_label() {
    assert!(page_html().contains("Config"));
}

#[test]
fn page_shows_step_progress_last_label() {
    assert!(page_html().contains("Pronto"));
}

#[test]
fn page_shows_step1_card_tag() {
    assert!(page_html().contains("PASSO 1 — CONFIGURAÇÃO"));
}

#[test]
fn page_shows_coop_name_label() {
    assert!(page_html().contains("Nome da Cooperativa"));
}

#[test]
fn page_shows_coop_name_placeholder() {
    assert!(page_html().contains("Coop Solidária do Nordeste"));
}

#[test]
fn page_shows_founder_wallet_label() {
    assert!(page_html().contains("Carteira Fundadora"));
}

#[test]
fn page_shows_founder_wallet_short_form_in_output() {
    assert!(page_html().contains("0x1111…1111"));
}
#[test]
fn page_shows_admin2_label() {
    assert!(page_html().contains("Admin 2 — Carteira"));
}

#[test]
fn page_shows_admin3_label() {
    assert!(page_html().contains("Admin 3 — Carteira"));
}

#[test]
fn page_shows_threshold_value() {
    assert!(page_html().contains("2 de 3 administradores"));
}

#[test]
fn page_shows_threshold_hint() {
    assert!(page_html().contains("Fixo para esta versão"));
}

#[test]
fn page_shows_submit_button() {
    assert!(page_html().contains("PREPARAR DEPLOY"));
}

// Steps 2-6 must NOT leak into the step-1 render

#[test]
fn page_hides_access_code_step() {
    let html = page_html();
    assert!(!html.contains("PASSO 2 — CÓDIGO DE ACESSO"), "step 2 tag leaked");
    assert!(!html.contains("ASSINAR TX 1"),               "deploy button leaked");
}

#[test]
fn page_hides_waiting_deploy_step() {
    assert!(!page_html().contains("Aguardando confirmação do deploy"));
}

#[test]
fn page_hides_sign_init_step() {
    let html = page_html();
    assert!(!html.contains("PASSO 3 — INICIALIZAR MULTISIG"), "step 3 tag leaked");
    assert!(!html.contains("ASSINAR TX 2"),                   "init button leaked");
}

#[test]
fn page_hides_registering_step() {
    assert!(!page_html().contains("Registrando cooperativa na blockchain"));
}

#[test]
fn page_hides_done_step() {
    let html = page_html();
    assert!(!html.contains("COOPERATIVA CRIADA!"),  "done card leaked");
    assert!(!html.contains("IR PARA O DASHBOARD"), "dashboard link leaked");
}

// ── StepProgress ─────────────────────────────────────────────
//
// StepProgress is fully synchronous. We test every step by setting
// the signal before rendering — no async needed.

fn progress_at(step: CoopStep) -> String {
    render_to_string(move || {
        let (s, _) = signal(step);
        view! { <StepProgress step=s /> }
    })
}

#[test]
fn progress_renders_all_six_labels() {
    let html = progress_at(CoopStep::Form);
    for label in ["Config", "Código", "Deploy", "Init", "Registro", "Pronto"] {
        assert!(html.contains(label), "missing badge label: {label}");
    }
}

#[test]
fn progress_form_step_one_badge_filled() {
    assert_eq!(progress_at(CoopStep::Form).matches("badge-filled-yellow").count(), 1);
}

#[test]
fn progress_access_code_step_two_badges_filled() {
    assert_eq!(progress_at(CoopStep::AccessCode).matches("badge-filled-yellow").count(), 2);
}

#[test]
fn progress_waiting_deploy_step_three_badges_filled() {
    assert_eq!(progress_at(CoopStep::WaitingDeploy).matches("badge-filled-yellow").count(), 3);
}

#[test]
fn progress_sign_init_step_four_badges_filled() {
    assert_eq!(progress_at(CoopStep::SignInit).matches("badge-filled-yellow").count(), 4);
}

#[test]
fn progress_registering_step_five_badges_filled() {
    assert_eq!(progress_at(CoopStep::Registering).matches("badge-filled-yellow").count(), 5);
}

#[test]
fn progress_done_step_all_six_badges_filled() {
    assert_eq!(progress_at(CoopStep::Done).matches("badge-filled-yellow").count(), 6);
}

// ── Error alert behavior ──────────────────────────────────────
//
// The CreateCoopPage owns a private `error` signal that drives a
// conditional Alert. We can't poke that signal from outside (it's
// component-local), but we CAN verify the initial render is clean —
// which is what users see on a fresh visit.

#[test]
fn page_does_not_show_error_alert_initially() {
    // Empty `error` signal → the (!e.is_empty()).then(...) branch
    // returns None, so no alert-error class should be in the HTML.
    assert!(!page_html().contains("alert-error"));
}

#[test]
fn page_does_not_show_error_alert_icon_initially() {
    // The Alert component prefixes an X icon for AlertKind::Error.
    // If our conditional fails, that icon would leak in. Belt-and-suspenders
    // version of the test above.
    let html = page_html();
    let error_alert_present = html.contains("alert-error") || html.contains("✕");
    assert!(!error_alert_present);
}

// ── Why the "clear-on-forward, keep-on-backward" rule isn't
//    asserted in SSR ────────────────────────────────────────────
//
// The directional `advance(next)` helper compares step indices and
// only clears `error` when next > current. Validating that requires
// driving the page through real transitions:
//   1. dispatch privy_tx_error  → error set, step rolls back
//   2. user retries             → forward transition clears error
// Step 1 needs a CustomEvent dispatch + listener round-trip, which
// is browser-only. The directional logic itself is straightforward
// arithmetic on the CoopStep enum — covered by code review, not SSR.