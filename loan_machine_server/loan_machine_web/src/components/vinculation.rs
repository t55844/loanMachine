// src/components/vinculation.rs
//
// Two-step form:
//   1. IdentifyCard  — user fills in coop id + access code.
//   2. TxStatusCard  — pending → complete | failed, driven by bridge events.
//
// The confirm-and-sign step is delegated to the app-wide `<GasModal />`
// (mounted in `app.rs`): when the server returns a bundle, this form
// opens the modal via `use_gas_modal()`.  The modal fires `on_confirm`
// when the user accepts; that's where we set tx_status → Pending and
// hand the bundle to `privy_bridge::send_tx`.

use leptos::prelude::*;
use leptos::ev::SubmitEvent;

use crate::components::ui::*;
use crate::components::gas_modal::{use_gas_modal, GasEstimate, GasModalRequest};
use crate::wallet_auth::privy_bridge::{self, TxOutcome};
use crate::server_fns::vinculation::prepare_first_vinculation;

// ── STATE TYPES ───────────────────────────────────────────────

#[derive(Clone, PartialEq)]
pub enum TxStatus {
    Idle,
    Pending,
    Complete(String), // tx hash
    Failed(String),   // error message
}

#[derive(Clone)]
struct FormFields {
    coop_id: String,
}

// ── PUBLIC ENTRY POINT ───────────────────────────────────────

#[component]
pub fn FirstVinculationForm(
    on_success: impl Fn() + 'static,
) -> impl IntoView {
    let prepare = Action::new(move |args: &FormFields| {
        let a = args.clone();
        async move {
            prepare_first_vinculation(a.coop_id).await
        }
    });

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

    let (tx_status, set_tx_status) = signal(TxStatus::Idle);

    privy_bridge::on_tx_outcome(move |outcome|
        match outcome {
        TxOutcome::Complete(hash) => set_tx_status.set(TxStatus::Complete(hash)),
        TxOutcome::Failed(err) => set_tx_status.set(TxStatus::Failed(err)),
    });

    Effect::new(move |_| {
        if matches!(tx_status.get(), TxStatus::Complete(_)) {
            on_success();
        }
    });

    let set_gas_modal = use_gas_modal();
    Effect::new(move |_| {
        if tx_status.get() != TxStatus::Idle {
            return;
        }
        let Some(b) = bundle.get() else { return; };

        let bundle_for_confirm = b.clone();
        set_gas_modal.set(Some(GasModalRequest {
            title: "SIGN LINKING".into(),
            estimates: vec![GasEstimate {
                label: "Link wallet to contract".into(),
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
            on_cancel: None,
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
    let (coop_id, set_coop_id) = signal(String::new());

    let on_prepare = move |ev: SubmitEvent| {
        ev.prevent_default();
        on_submit.run(FormFields { coop_id: coop_id.get() });
    };

    view! {
        <div class="t-center">
            <h1 class="t-display-lg t-yellow">"LINK YOUR WALLET"</h1>
            <p class="t-mono-sm t-muted mt-4">
                "Link your wallet to the cooperative on-chain."
            </p>
        </div>

        <Card variant=CardVariant::Yellow tag="STEP 01 — LINK" hover=false>
            <form
                on:submit=on_prepare
                style="display:flex; flex-direction:column; gap:var(--sp-6); margin-top:var(--sp-4)"
            >
                <TextInput
                    label="Cooperative ID"
                    placeholder="0xabc123..."
                    hint="The bytes32 identifier of your cooperative"
                    value=coop_id
                    set_value=set_coop_id
                    error=error
                />

                <Button
                    variant=BtnVariant::Primary
                    size=BtnSize::Lg
                    full_width=true
                    loading=loading
                >
                    "PREPARE LINKING"
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
                        "Waiting for signature and confirmation..."
                    </span>
                </div>
            </Card>
        }
        .into_any(),

        TxStatus::Complete(hash) => view! {
            <Card variant=CardVariant::Gold tag="LINKING SUBMITTED" hover=false>
                <div class="flex-col gap-4 mt-4">
                    <Alert kind=AlertKind::Success>
                        "Transaction submitted successfully!"
                    </Alert>
                    <div class="stat-block">
                        <span class="stat-label">"Transaction Hash"</span>
                        <HashDisplay value=hash />
                    </div>
                    <p class="t-mono-xs t-muted">
                        "Wait for confirmation on the blockchain. \
                         This may take 10–30 seconds."
                    </p>
                </div>
            </Card>
        }
        .into_any(),

        TxStatus::Failed(err) => view! {
            <Card hover=false>
                <div class="flex-col gap-4 mt-4">
                    <Alert kind=AlertKind::Error>
                        "Failed to submit transaction."
                    </Alert>
                    <p class="t-mono-sm t-muted">{err}</p>
                    <Button
                        variant=BtnVariant::Ghost
                        on_click=Box::new(move || on_retry.run(()))
                    >
                        "TRY AGAIN"
                    </Button>
                </div>
            </Card>
        }
        .into_any(),
    }
}
