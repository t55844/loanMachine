// loan_machine_web/src/components/vinculation_test.rs
//
// SSR snapshot tests for the vinculation components.
//
// These only cover the INITIAL RENDER of each component.
// Reactivity (signal updates, events, spawn_local futures) does not
// tick in SSR rendering — those cases need wasm-bindgen-test in a
// real browser. See `vinculation_wasm_test.rs` for that layer.


use leptos::prelude::*;
use crate::components::vinculation::{VinculationGate, FirstVinculationForm};
use crate::components::tests_helper::fake_wallet;
// ── Helpers ──────────────────────────────────────────────────

/// Render any view to an HTML string inside an Owner scope.
/// The Owner is required because signals register with it at creation.
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

// ── VinculationGate ──────────────────────────────────────────
//
// The gate immediately spawns get_wallet_coop() via spawn_local,
// but during SSR that future is scheduled and never polled to
// completion — so the first paint always shows the Checking state.

async fn gate_html() -> String {
    any_spawner::Executor::init_tokio().ok();
    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        render_to_string(|| view! {
            <VinculationGate smart_wallet=fake_wallet()>
                <div>"test-children-marker"</div>
            </VinculationGate>
        })
    }).await
}
#[tokio::test]
async fn async_gate_shows_spinner_on_initial_render() {
    assert!(gate_html().await.contains("spinner"));
}

#[tokio::test]
async fn async_gate_shows_checking_message_on_initial_render() {
    assert!(gate_html().await.contains("Verificando vínculo"));
}

#[tokio::test]
async fn async_gate_hides_children_on_initial_render() {
    // Children only render in the Vinculated branch,
    // which isn't reachable in SSR without a running server fn
    assert!(!gate_html().await.contains("test-children-marker"));
}

#[tokio::test]
async fn async_gate_hides_form_on_initial_render() {
    assert!(!gate_html().await.contains("VINCULE SUA CARTEIRA"));
}

// ── FirstVinculationForm ─────────────────────────────────────
//
// This one is fully synchronous — no spawn_local on mount,
// so SSR render = real render.

fn form_html() -> String {
    render_to_string(|| view! {
        <FirstVinculationForm
            smart_wallet=fake_wallet()
            on_success=|_| {}
        />
    })
}

// Title + subtitle
#[test]
fn form_shows_title() {
    assert!(form_html().contains("VINCULE SUA CARTEIRA"));
}

#[test]
fn form_shows_updated_subtitle() {
    // Text was changed from "ID de membro" to "documento"
    assert!(form_html().contains("Conecte seu documento"));
}

// Card 1 — identification
#[test]
fn form_shows_step_01_card() {
    assert!(form_html().contains("PASSO 01"));
}

#[test]
fn form_shows_both_doc_kind_toggles() {
    let html = form_html();
    assert!(html.contains(">CPF<"),  "CPF toggle button missing");
    assert!(html.contains(">CNPJ<"), "CNPJ toggle button missing");
}

#[test]
fn form_defaults_to_cpf_mode() {
    let html = form_html();
    // CPF button has the active class on first render
    assert!(html.contains("toggle-btn-active"));
    // CPF placeholder is present (14 chars, one hyphen)
    assert!(html.contains("000.000.000-00"));
    // CNPJ placeholder is NOT the current placeholder
    assert!(!html.contains(r#"placeholder="00.000.000/0000-00""#));
}

#[test]
fn form_shows_coop_id_input() {
    assert!(form_html().contains("ID da Cooperativa"));
}

#[test]
fn form_shows_access_code_input() {
    assert!(form_html().contains("Código de Acesso"));
}

#[test]
fn form_shows_prepare_button() {
    assert!(form_html().contains("PREPARAR VINCULAÇÃO"));
}

// Cards 2 and 3 — bundle+tx status — must be hidden initially
#[test]
fn form_hides_step_02_initially() {
    let html = form_html();
    assert!(!html.contains("PASSO 02"),              "step 02 tag leaked");
    assert!(!html.contains("ASSINAR COM SUA CARTEIRA"), "sign button leaked");
}

#[test]
fn form_hides_tx_status_initially() {
    let html = form_html();
    assert!(!html.contains("Aguardando assinatura"),   "pending state leaked");
    assert!(!html.contains("VINCULAÇÃO ENVIADA"),      "complete state leaked");
    assert!(!html.contains("Falha ao enviar transação"), "failed state leaked");
}

#[test]
fn form_input_enforces_cpf_max_length() {
    let html = form_html();
    // maxlength="14" is bound to doc_kind.max_input_len()
    // Default is CPF → 14
    assert!(html.contains(r#"maxlength="14""#));
}