// src/components/vinculation.rs
//
// Two-step form (the third step lives in `GasModal`):
//   1. IdentifyCard  — user fills in document + coop id + access code.
//   2. TxStatusCard  — pending → complete | failed, driven by bridge events.
//
// The confirm-and-sign step is delegated to the app-wide `<GasModal />`
// (mounted in `app.rs`): when the server returns a bundle, this form
// opens the modal via `use_gas_modal()`.  The modal fires `on_confirm`
// when the user accepts; that's where we set tx_status → Pending and
// hand the bundle to `privy_bridge::send_tx`.
//
// All JS interop lives behind `crate::wallet_auth::privy_bridge`.
// The CPF/CNPJ input block lives behind `ui::DocumentInput` — no
// wasm_bindgen / web_sys imports in this file anymore.

use leptos::prelude::*;
use leptos::ev::SubmitEvent;

use crate::components::ui::*;
use crate::components::gas_modal::{use_gas_modal, GasEstimate, GasModalRequest};
use crate::wallet_auth::privy_bridge::{self, TxOutcome};
use loan_machine_models::requests::DocKind;
use crate::components::cpf_cnpj::doc_input_snipet::{DocumentInput};
use crate::server_fns::vinculation::prepare_first_vinculation;

// ── STATE TYPES ───────────────────────────────────────────────

/// UI status of the on-chain vinculation tx.
///
/// Note the relationship with `privy_bridge::TxOutcome`: the bridge
/// can only produce Complete/Failed (those are the events JS emits).
/// We add Idle and Pending here as UI-only states that the bridge
/// has no business knowing about.  Mapping happens inside the form.
#[derive(Clone, PartialEq)]
pub enum TxStatus {
    Idle,
    Pending,
    Complete(String), // tx hash
    Failed(String),   // error message
}

/// The four fields the form collects.  These are also the four
/// arguments the server fn takes, so we use this struct directly as
/// the `Action` input.
#[derive(Clone)]
struct FormFields {
    kind: DocKind,
    document: String,
    coop_id: String,
    access_code: String,
}

// ── PUBLIC ENTRY POINT ───────────────────────────────────────

#[component]
pub fn FirstVinculationForm(
    /// Fires once the tx is confirmed on-chain.  Typically used by
    /// the caller to navigate away from the form (e.g. redirect home).
    on_success: impl Fn() + 'static,
) -> impl IntoView {
    // ── Server mutation → `Action` ─────────────────────────────
    let prepare = Action::new(move |args: &FormFields| {
        let a = args.clone();
        async move {
            prepare_first_vinculation(a.kind, a.document, a.coop_id, a.access_code).await
        }
    });

    // ── Derived signals from the action ────────────────────────
    let loading = prepare.pending();
    let error_text = Signal::derive(move ||
        match prepare.value().get() {
        Some(Err(e)) => e.to_string(),
        _ => String::new(),
    });
    let bundle = Signal::derive(move || match prepare.value().get() {
        Some(Ok(b)) => Some(b),
        _ => None,
    });

    // ── Tx status state machine ────────────────────────────────
    let (tx_status, set_tx_status) = signal(TxStatus::Idle);

    // Subscribe to bridge events. The closure maps the bridge's
    // TxOutcome (Complete/Failed) into our richer TxStatus.
    // Cleanup is automatic on unmount — the bridge handles it.
    privy_bridge::on_tx_outcome(move |outcome|
        match outcome {
        TxOutcome::Complete(hash) => set_tx_status.set(TxStatus::Complete(hash)),
        TxOutcome::Failed(err) => set_tx_status.set(TxStatus::Failed(err)),
    });

    // Bridge tx confirmation → caller's `on_success`.
    Effect::new(move |_| {
        if matches!(tx_status.get(), TxStatus::Complete(_)) {
            on_success();
        }
    });

    // ── Open the gas modal when a fresh bundle is ready ────────
    let set_gas_modal = use_gas_modal();
    Effect::new(move |_| {
        if tx_status.get() != TxStatus::Idle {
            return;
        }
        let Some(b) = bundle.get() else { return; };

        let bundle_for_confirm = b.clone();
        set_gas_modal.set(Some(GasModalRequest {
            title: "ASSINAR VINCULAÇÃO".into(),
            estimates: vec![GasEstimate {
                label: "Vincular carteira ao contrato".into(),
                gas_hex: b.gas_join.clone(),
            }],
            on_confirm: Callback::new(move |()| {
                set_tx_status.set(TxStatus::Pending);
                privy_bridge::send_tx(
                    &bundle_for_confirm.loan_machine_address,
                    &bundle_for_confirm.join_calldata,
                    None,
                );
            }),
        }));
    });

    view! {
        <div class="flex-col gap-8">
            <IdentifyCard
                error=error_text
                loading=loading
                on_submit=Callback::new(move |fields: FormFields| {
                    prepare.dispatch(fields);
                })
            />

            <TxStatusCard
                tx_status=tx_status
                on_retry=Callback::new(move |()| set_tx_status.set(TxStatus::Idle))
            />
        </div>
    }
}

// ── STEP COMPONENTS ──────────────────────────────────────────

#[component]
fn IdentifyCard(
    on_submit: Callback<FormFields>,
    #[prop(into)] error: Signal<String>,
    #[prop(into)] loading: Signal<bool>,
) -> impl IntoView {
    // Signals owned by the form.  The toggle-and-clear behavior + the
    // input filter live inside DocumentInput — nothing to wire here.
    let (doc_kind,   set_doc_kind)    = signal(DocKind::Cpf);
    let (document,   set_document)    = signal(String::new());
    let (coop_id,    set_coop_id)     = signal(String::new());
    let (access_code, set_access_code) = signal(String::new());

    let on_prepare = move |ev: SubmitEvent| {
        ev.prevent_default();
        on_submit.run(FormFields {
            kind:        doc_kind.get(),
            document:    document.get(),
            coop_id:     coop_id.get(),
            access_code: access_code.get(),
        });
    };

    view! {
        <div class="t-center">
            <h1 class="t-display-lg t-yellow">"VINCULE SUA CARTEIRA"</h1>
            <p class="t-mono-sm t-muted mt-4">
                "Conecte seu documento ao seu endereço blockchain."
            </p>
        </div>

        <Card variant=CardVariant::Yellow tag="PASSO 01 — IDENTIFICAÇÃO" hover=false>
            <form
                on:submit=on_prepare
                style="display:flex; flex-direction:column; gap:var(--sp-6); margin-top:var(--sp-4)"
            >
                <DocumentInput
                    doc_kind set_doc_kind
                    document set_document
                />

                <TextInput
                    label="ID da Cooperativa"
                    placeholder="0xabc123..."
                    hint="O identificador bytes32 da sua cooperativa"
                    value=coop_id
                    set_value=set_coop_id
                />

                <TextInput
                    label="Código de Acesso"
                    placeholder="Fornecido pelo administrador"
                    value=access_code
                    set_value=set_access_code
                    error=error
                />

                <Button
                    variant=BtnVariant::Primary
                    size=BtnSize::Lg
                    full_width=true
                    loading=loading
                >
                    "PREPARAR VINCULAÇÃO"
                </Button>
            </form>
        </Card>
    }
}

#[component]
fn TxStatusCard(
    tx_status: ReadSignal<TxStatus>,
    on_retry: Callback<()>,
) -> impl IntoView {
    move || match tx_status.get() {
        TxStatus::Idle => ().into_any(),

        TxStatus::Pending => view! {
            <Card hover=false>
                <div class="flex-center gap-4" style="padding: var(--sp-8) 0">
                    <span class="spinner"></span>
                    <span class="t-mono-sm t-muted">
                        "Aguardando assinatura e confirmação..."
                    </span>
                </div>
            </Card>
        }
        .into_any(),

        TxStatus::Complete(hash) => view! {
            <Card variant=CardVariant::Gold tag="VINCULAÇÃO ENVIADA" hover=false>
                <div class="flex-col gap-4 mt-4">
                    <Alert kind=AlertKind::Success>
                        "Transação enviada com sucesso!"
                    </Alert>
                    <div class="stat-block">
                        <span class="stat-label">"Transaction Hash"</span>
                        <HashDisplay value=hash />
                    </div>
                    <p class="t-mono-xs t-muted">
                        "Aguarde a confirmação na blockchain. \
                         Isso pode levar 10–30 segundos."
                    </p>
                </div>
            </Card>
        }
        .into_any(),

        TxStatus::Failed(err) => view! {
            <Card hover=false>
                <div class="flex-col gap-4 mt-4">
                    <Alert kind=AlertKind::Error>
                        "Falha ao enviar transação."
                    </Alert>
                    <p class="t-mono-sm t-muted">{err}</p>
                    <Button
                        variant=BtnVariant::Ghost
                        on_click=Box::new(move || on_retry.run(()))
                    >
                        "TENTAR NOVAMENTE"
                    </Button>
                </div>
            </Card>
        }
        .into_any(),
    }
}