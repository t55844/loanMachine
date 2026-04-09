// vinculation_test.rs

use leptos::prelude::*;
use crate::components::vinculation::{VinculationGate, FirstVinculationForm};

use std::sync::OnceLock;

static EXECUTOR: OnceLock<()> = OnceLock::new();

fn init_executor() {
    EXECUTOR.get_or_init(|| {
        any_spawner::Executor::init_tokio().expect("executor init failed");
    });
}

fn render_gate() -> String {
    init_executor();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let local = tokio::task::LocalSet::new();

    local.block_on(&rt, async {
        let owner = Owner::new();
        owner.with(|| {
            view! {
                <VinculationGate smart_wallet="0x1234".to_string()>
                    <div>"children"</div>
                </VinculationGate>
            }.to_html()
        })
    })
}

#[test]
fn gate_initial_state_shows_spinner() {
    assert!(render_gate().contains("spinner"));
}

#[test]
fn gate_initial_state_shows_checking_message() {
    assert!(render_gate().contains("Verificando vínculo..."));
}

#[test]
fn gate_initial_state_does_not_show_children() {
    // children should be hidden until Vinculated state
    assert!(!render_gate().contains("children"));
}

#[test]
fn gate_initial_state_does_not_show_form() {
    // form only appears in NeedsVinculation state
    assert!(!render_gate().contains("VINCULE SUA CARTEIRA"));
}

// ── FirstVinculationForm ──────────────────────────────────────────────────
// Initial render is fully deterministic — no async involved

fn render_form() -> String {
    let owner = Owner::new();
    owner.with(|| {
        view! {
            <FirstVinculationForm
                smart_wallet="0x1234".to_string()
                on_success=|_| {}
            />
        }.to_html()
    })
}

#[test]
fn form_shows_title() {
    assert!(render_form().contains("VINCULE SUA CARTEIRA"));
}

#[test]
fn form_shows_subtitle() {
    assert!(render_form().contains("Conecte seu ID de membro ao seu endereço blockchain."));
}

#[test]
fn form_shows_step_01_card() {
    assert!(render_form().contains("PASSO 01"));
}

#[test]
fn form_shows_member_id_input() {
    assert!(render_form().contains("ID do Membro"));
}

#[test]
fn form_shows_coop_id_input() {
    assert!(render_form().contains("ID da Cooperativa"));
}

#[test]
fn form_shows_access_code_input() {
    assert!(render_form().contains("Código de Acesso"));
}

#[test]
fn form_shows_prepare_button() {
    assert!(render_form().contains("PREPARAR VINCULAÇÃO"));
}

#[test]
fn form_does_not_show_step_02_initially() {
    // card 2 only appears when bundle signal is Some
    assert!(!render_form().contains("PASSO 02"));
    assert!(!render_form().contains("ASSINAR COM SUA CARTEIRA"));
}

#[test]
fn form_does_not_show_tx_status_initially() {
    // tx status card only appears after sign is clicked
    assert!(!render_form().contains("Aguardando assinatura"));
    assert!(!render_form().contains("VINCULAÇÃO ENVIADA"));
    assert!(!render_form().contains("Falha ao enviar transação"));
}