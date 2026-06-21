// loan_machine_web/src/components/create_coop_steps.rs
//
// Presentational step panels for the create-coop flow.
// No signals, no effects, no server calls — only props in, view out.

use leptos::prelude::*;
use loan_machine_models::responses::CoopRegistrationResult;
use loan_machine_models::wallet_address::WalletAddress;

use crate::components::ui::*;
use crate::components::create_coop::create_coop::CoopStep;
use leptos_router::components::A;

// ── FormStep ──────────────────────────────────────────────────

#[component]
pub fn FormStep(
    name:       ReadSignal<String>,
    set_name:   WriteSignal<String>,
    admin2:     ReadSignal<String>,
    set_admin2: WriteSignal<String>,
    admin3:     ReadSignal<String>,
    set_admin3: WriteSignal<String>,
    name_err:   ReadSignal<String>,
    admin2_err: ReadSignal<String>,
    admin3_err: ReadSignal<String>,
    #[prop(into)] loading: Signal<bool>,
    founder_wallet: WalletAddress,
    on_submit: Box<dyn Fn() + Send + Sync>,
) -> impl IntoView {
    view! {
        <Card>
            <span class="card-tag">"STEP 1 — CONFIGURATION"</span>
            <div class="flex-col gap-6" style="margin-top: var(--sp-6)">

                <TextInput
                    label="Cooperative Name"
                    placeholder="E.g.: Northeast Solidarity Coop"
                    value=name set_value=set_name
                    error=Signal::derive(move || name_err.get())
                />

                <div class="form-group">
                    <label class="form-label">"Founder Wallet (you)"</label>
                    <div class="hash-display">{founder_wallet.short()}</div>
                    <span class="form-hint">
                        "Automatically filled with the connected wallet"
                    </span>
                </div>

                <TextInput
                    label="Admin 2 — Wallet"
                    placeholder="0x0000...0000"
                    hint="Second administrator of the cooperative"
                    value=admin2 set_value=set_admin2
                    error=Signal::derive(move || admin2_err.get())
                />

                <TextInput
                    label="Admin 3 — Wallet"
                    placeholder="0x0000...0000"
                    hint="Third administrator of the cooperative"
                    value=admin3 set_value=set_admin3
                    error=Signal::derive(move || admin3_err.get())
                />

                <div class="form-group">
                    <label class="form-label">"Approval Threshold"</label>
                    <div class="hash-display" style="color: var(--c-yellow)">
                        "2 of 3 administrators"
                    </div>
                    <span class="form-hint">"Fixed for this version"</span>
                </div>

                <Button
                    variant=BtnVariant::Primary full_width=true
                    loading=Signal::derive(move || loading.get())
                    on_click=on_submit
                >
                    "PREPARE DEPLOY"
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
            <span class="card-tag">"STEP 3 — INITIALIZE MULTISIG"</span>
            <div class="flex-col gap-6" style="margin-top: var(--sp-6)">

                <Alert kind=AlertKind::Success>
                    "Contract deployed! Now initialize the multisig to activate the administrators."
                </Alert>

                <div class="stat-block">
                    <span class="stat-label">"Contract Address"</span>
                    <HashDisplay value=contract_address />
                </div>

                <Button variant=BtnVariant::Primary full_width=true on_click=on_sign>
                    "SIGN TX 2 — INITIALIZE"
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
                <span class="card-tag">"COOPERATIVE CREATED!"</span>
                <div class="flex-col gap-6" style="margin-top: var(--sp-6)">

                    <Alert kind=AlertKind::Success>
                        "Cooperative registered successfully. Share the ID below \
                        with members so they can link up."
                    </Alert>

                    <div class="stat-block">
                        <span class="stat-label">"Cooperative ID"</span>
                        <HashDisplay value=r.coop_id_hex.clone() />
                    </div>
                    <div class="stat-block">
                        <span class="stat-label">"Contract Address"</span>
                        <HashDisplay value=r.loan_machine_address />
                    </div>
                    <div class="stat-block">
                        <span class="stat-label">"Registration TX"</span>
                        <HashDisplay value=r.registration_tx_hash />
                    </div>

                    <A href=format!("/cooperatives/{}", r.coop_id_hex.clone()) attr:class="card-link">
                        <Button variant=BtnVariant::Ghost full_width=true>
                            <span class="stat-label">"Control Panel"</span>
                        </Button>
                    </A>
                </div>
            </Card>
        }.into_any(),

        None => view! {
            <Alert kind=AlertKind::Error>"Unexpected error: result not found."</Alert>
        }.into_any(),
    }
}

// ── StepProgress ──────────────────────────────────────────────

#[component]
pub fn StepProgress(step: ReadSignal<CoopStep>) -> impl IntoView {
    let labels = ["Config", "Deploy", "Init", "Registry", "Done"];

    let current = move || match step.get() {
        CoopStep::Form          => 0usize,
        CoopStep::WaitingDeploy => 1,
        CoopStep::SignInit      => 2,
        CoopStep::Registering   => 3,
        CoopStep::Done          => 4,
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