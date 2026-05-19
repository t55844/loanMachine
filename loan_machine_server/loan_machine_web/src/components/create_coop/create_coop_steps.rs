// loan_machine_web/src/components/create_coop_steps.rs
//
// Presentational step panels for the create-coop flow.
// No signals, no effects, no server calls — only props in, view out.
// All callbacks arrive as Box<dyn Fn() + Send + Sync> from CreateCoopPage.
//
// CHANGES vs prior version:
//   • FormStep now takes the (doc_kind, document) signal pair instead
//     of a single cpf_cnpj String.
//   • The hand-rolled CPF/CNPJ input block (the part you tried and
//     failed) is replaced by <DocumentInput .../>.  All toggle and
//     filter logic lives inside that component now.
//   • New prop: document_err (mirrors the other *_err signals).

use leptos::prelude::*;
use loan_machine_models::requests::DocKind;
use loan_machine_models::responses::CoopRegistrationResult;
use loan_machine_models::wallet_address::WalletAddress;
use crate::components::cpf_cnpj::doc_input_snipet::{DocumentInput};

use crate::components::ui::*;
use crate::components::create_coop::create_coop::CoopStep;
use leptos_router::components::A;

// ── FormStep ──────────────────────────────────────────────────

#[component]
pub fn FormStep(
    name:         ReadSignal<String>,
    set_name:     WriteSignal<String>,
    doc_kind:     ReadSignal<DocKind>,
    set_doc_kind: WriteSignal<DocKind>,
    document:     ReadSignal<String>,
    set_document: WriteSignal<String>,
    admin2:       ReadSignal<String>,
    set_admin2:   WriteSignal<String>,
    admin3:       ReadSignal<String>,
    set_admin3:   WriteSignal<String>,
    name_err:     ReadSignal<String>,
    document_err: ReadSignal<String>,
    admin2_err:   ReadSignal<String>,
    admin3_err:   ReadSignal<String>,
    #[prop(into)] loading: Signal<bool>,
    founder_wallet: WalletAddress,
    on_submit: Box<dyn Fn() + Send + Sync>,
) -> impl IntoView {
    view! {
        <Card>
            <span class="card-tag">"PASSO 1 — CONFIGURAÇÃO"</span>
            <div class="flex-col gap-6" style="margin-top: var(--sp-6)">

                <TextInput
                    label="Nome da Cooperativa"
                    placeholder="Ex: Coop Solidária do Nordeste"
                    value=name set_value=set_name
                    error=Signal::derive(move || name_err.get())
                />

                // Founder document — same reusable component as vinculation.
                // Used to derive the founder's memberId server-side, which
                // is then baked into the initializeMultisig calldata so the
                // founder is a full member the moment the contract is live.
                <DocumentInput
                    doc_kind set_doc_kind
                    document set_document
                    error=Signal::derive(move || document_err.get())
                />

                <div class="form-group">
                    <label class="form-label">"Carteira Fundadora (você)"</label>
                    <div class="hash-display">{founder_wallet.short()}</div>
                    <span class="form-hint">
                        "Preenchido automaticamente pela carteira conectada"
                    </span>
                </div>

                <TextInput
                    label="Admin 2 — Carteira"
                    placeholder="0x0000...0000"
                    hint="Segundo administrador da cooperativa"
                    value=admin2 set_value=set_admin2
                    error=Signal::derive(move || admin2_err.get())
                />

                <TextInput
                    label="Admin 3 — Carteira"
                    placeholder="0x0000...0000"
                    hint="Terceiro administrador da cooperativa"
                    value=admin3 set_value=set_admin3
                    error=Signal::derive(move || admin3_err.get())
                />

                <div class="form-group">
                    <label class="form-label">"Threshold de Aprovação"</label>
                    <div class="hash-display" style="color: var(--c-yellow)">
                        "2 de 3 administradores"
                    </div>
                    <span class="form-hint">"Fixo para esta versão"</span>
                </div>

                <Button
                    variant=BtnVariant::Primary full_width=true
                    loading=Signal::derive(move || loading.get())
                    on_click=on_submit
                >
                    "PREPARAR DEPLOY"
                </Button>
            </div>
        </Card>
    }
}

// ── AccessCodeStep ────────────────────────────────────────────

#[component]
pub fn AccessCodeStep(
    access_code: String,
    code_saved: ReadSignal<bool>,
    set_code_saved: WriteSignal<bool>,
    on_sign: Box<dyn Fn() + Send + Sync>,
) -> impl IntoView {
    view! {
        <Card variant=CardVariant::Yellow>
            <span class="card-tag">"PASSO 2 — CÓDIGO DE ACESSO"</span>
            <div class="flex-col gap-6" style="margin-top: var(--sp-6)">

                <Alert kind=AlertKind::Warning>
                    "ATENÇÃO: Salve este código agora. Ele não será exibido
                    novamente. Sem ele, nenhum membro poderá entrar na cooperativa."
                </Alert>

                <div class="flex-col flex-center gap-4">
                    <span class="t-mono-xs t-muted" style="text-transform: uppercase; letter-spacing: 0.1em">
                        "Código de Acesso"
                    </span>
                    <div style="
                        font-family: var(--font-display);
                        font-size: 2.8rem;
                        letter-spacing: 0.18em;
                        color: var(--c-yellow);
                        background: rgba(240,204,0,0.07);
                        border: 1px solid rgba(240,204,0,0.3);
                        border-radius: var(--radius-lg);
                        padding: var(--sp-6) var(--sp-8);
                        text-align: center;
                    ">
                        {access_code}
                    </div>
                </div>

                <div style="display: flex; align-items: center; gap: var(--sp-3)">
                    <input
                        type="checkbox"
                        id="code-saved"
                        style="width: 18px; height: 18px; accent-color: var(--c-yellow); cursor: pointer; flex-shrink: 0"
                        on:change=move |ev| set_code_saved.set(event_target_checked(&ev))
                    />
                    <label for="code-saved" class="t-mono-sm t-bright" style="cursor: pointer">
                        "Salvei o código de acesso em lugar seguro"
                    </label>
                </div>

                <Button
                    variant=BtnVariant::Primary full_width=true
                    disabled=Signal::derive(move || !code_saved.get())
                    on_click=on_sign
                >
                    "ASSINAR TX 1 — DEPLOY DO CONTRATO"
                </Button>
            </div>
        </Card>
    }
}

// ── WaitingStep ───────────────────────────────────────────────

#[component]
pub fn WaitingStep(label: &'static str, hint: &'static str) -> impl IntoView {
    view! {
        <Card>
            <div class="flex-col flex-center gap-6" style="padding: var(--sp-12) 0">
                <span class="spinner" style="width: 40px; height: 40px; border-width: 3px" />
                <div class="t-center flex-col gap-2">
                    <p class="t-mono-sm t-bright">{label}</p>
                    <p class="t-mono-xs t-muted">{hint}</p>
                </div>
            </div>
        </Card>
    }
}

// ── SignInitStep ──────────────────────────────────────────────

#[component]
pub fn SignInitStep(contract_address: String, on_sign: Box<dyn Fn() + Send + Sync>) -> impl IntoView {
    view! {
        <Card variant=CardVariant::Gold>
            <span class="card-tag">"PASSO 3 — INICIALIZAR MULTISIG"</span>
            <div class="flex-col gap-6" style="margin-top: var(--sp-6)">

                <Alert kind=AlertKind::Success>
                    "Contrato implantado! Agora inicialize o multisig para ativar os administradores."
                </Alert>

                <div class="stat-block">
                    <span class="stat-label">"Endereço do Contrato"</span>
                    <HashDisplay value=contract_address />
                </div>

                <Button variant=BtnVariant::Primary full_width=true on_click=on_sign>
                    "ASSINAR TX 2 — INICIALIZAR"
                </Button>
            </div>
        </Card>
    }
}

// ── DoneStep ──────────────────────────────────────────────────

#[component]
pub fn DoneStep(reg_result: ReadSignal<Option<CoopRegistrationResult>>) -> impl IntoView {
    move || match reg_result.get() {
        Some(r) => view! {
            <Card variant=CardVariant::Yellow>
                <span class="card-tag">"COOPERATIVA CRIADA!"</span>
                <div class="flex-col gap-6" style="margin-top: var(--sp-6)">

                    <Alert kind=AlertKind::Success>
                        "Cooperativa registrada com sucesso. Compartilhe o ID
                        abaixo com os membros para que eles possam se vincular."
                    </Alert>

                    <div class="stat-block">
                        <span class="stat-label">"ID da Cooperativa"</span>
                        <HashDisplay value=r.coop_id_hex.clone() />
                    </div>
                    <div class="stat-block">
                        <span class="stat-label">"Endereço do Contrato"</span>
                        <HashDisplay value=r.loan_machine_address />
                    </div>
                    <div class="stat-block">
                        <span class="stat-label">"TX de Registro"</span>
                        <HashDisplay value=r.registration_tx_hash />
                    </div>

                    <A href=format!("/cooperatives/{}", r.coop_id_hex.clone()) attr:class="card-link">
                        <Button variant=BtnVariant::Ghost full_width=true>
                            <span class="stat-label">"Painel de controle"</span>
                        </Button>
                    </A>
                </div>
            </Card>
        }.into_any(),

        None => view! {
            <Alert kind=AlertKind::Error>"Erro inesperado: resultado não encontrado."</Alert>
        }.into_any(),
    }
}

// ── StepProgress ──────────────────────────────────────────────

#[component]
pub fn StepProgress(step: ReadSignal<CoopStep>) -> impl IntoView {
    let labels = ["Config", "Código", "Deploy", "Init", "Registro", "Pronto"];

    let current = move || match step.get() {
        CoopStep::Form          => 0usize,
        CoopStep::AccessCode    => 1,
        CoopStep::WaitingDeploy => 2,
        CoopStep::SignInit      => 3,
        CoopStep::Registering   => 4,
        CoopStep::Done          => 5,
    };

    view! {
        <div style="display: flex; gap: var(--sp-2); justify-content: center; flex-wrap: wrap; margin-top: var(--sp-4)">
            {labels.iter().enumerate().map(|(i, label)| {
                view! {
                    <span
                        class=move || if i <= current() {
                            "badge badge-filled-yellow"
                        } else {
                            "badge badge-yellow"
                        }
                        style="font-size: 0.6rem"
                    >
                        {(i + 1).to_string()}". "{*label}
                    </span>
                }
            }).collect_view()}
        </div>
    }
}