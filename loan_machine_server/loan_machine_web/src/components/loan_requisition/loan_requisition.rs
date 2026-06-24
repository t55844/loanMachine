// src/components/loan_requisition/loan_requisition.rs
//
// Loan requisition form: amount + parameters + submit, then a single
// signature via the Privy bridge — `LoanMachine.createLoanRequisition`.
// The confirm-and-sign step is delegated to the app-wide `<GasModal />`.

use leptos::prelude::*;
use leptos::ev::SubmitEvent;

use crate::components::ui::*;
use crate::components::gas_modal::{use_gas_modal, GasEstimate, GasModalRequest};
use crate::wallet_auth::privy_bridge::{self, TxOutcome};
use crate::server_fns::loan_requisition::prepare_loan_requisition;
use loan_machine_models::responses::LoanRequisitionBundle;

// ── STATE TYPES ───────────────────────────────────────────────

#[derive(Clone, PartialEq)]
enum LoanTxStatus {
    Idle,
    Pending,
    Complete(String), // tx hash
    Failed(String),   // error message
}

// ── PUBLIC ENTRY POINT ───────────────────────────────────────

#[component]
pub fn LoanRequisitionForm(
    #[prop(into)] coop_id: String,
    on_tx_success: impl Fn() + 'static,
) -> impl IntoView {
    let (amount,        set_amount)        = signal(String::new());
    let (parcels_count, set_parcels_count) = signal(6u32);
    let (days_interval, set_days_interval) = signal(30u32);

    let prepare = Action::new({
        let coop_id = coop_id.clone();
        move |(raw_amount, parcels, interval): &(String, u32, u32)| {
            let coop_id  = coop_id.clone();
            let raw      = raw_amount.clone();
            let parcels  = *parcels;
            let interval = *interval;
            async move {
                prepare_loan_requisition(coop_id, raw, parcels, interval).await
            }
        }
    });

    let loading = prepare.pending();
    let prepare_error = Signal::derive(move || match prepare.value().get() {
        Some(Err(e)) => e.to_string(),
        _ => String::new(),
    });
    let bundle: Signal<Option<LoanRequisitionBundle>> = Signal::derive(move || {
        match prepare.value().get() {
            Some(Ok(b)) => Some(b),
            _ => None,
        }
    });

    let (tx_status, set_tx_status) = signal(LoanTxStatus::Idle);
    let set_gas_modal = use_gas_modal();

    Effect::new(move |_| {
        if tx_status.get() != LoanTxStatus::Idle {
            return;
        }
        if loading.get() { return; }
        let Some(b) = bundle.get() else { return; };

        let b = b.clone();
        set_gas_modal.set(Some(GasModalRequest {
            title: "SIGN LOAN REQUEST".into(),
            estimates: vec![GasEstimate {
                label: "Create loan requisition".into(),
                gas_hex: b.gas_hex.clone(),
            }],
            on_confirm: Callback::new(move |()| {
                set_tx_status.set(LoanTxStatus::Pending);
                privy_bridge::send_tx(&b.loan_machine_address, &b.calldata, Some(&b.gas_hex));
            }),
            on_cancel: Some(Callback::new(move |()| {
                set_tx_status.set(LoanTxStatus::Failed("Transaction cancelled.".into()));
            })),
        }));
    });

    privy_bridge::on_tx_outcome(move |outcome| {
        match (tx_status.get_untracked(), outcome) {
            (LoanTxStatus::Pending, TxOutcome::Complete(hash)) => {
                set_tx_status.set(LoanTxStatus::Complete(hash));
            }
            (LoanTxStatus::Pending, TxOutcome::Failed(err)) => {
                set_tx_status.set(LoanTxStatus::Failed(err));
            }
            _ => {}
        }
    });

    Effect::new(move |_| {
        if matches!(tx_status.get(), LoanTxStatus::Complete(_)) {
            on_tx_success();
        }
    });

    let parcels_error = Signal::derive(move || {
        let v = parcels_count.get();
        if v < 1 || v > 12 {
            "Number of installments must be between 1 and 12".to_string()
        } else {
            String::new()
        }
    });

    let interval_error = Signal::derive(move || {
        let v = days_interval.get();
        if v < 1 || v > 30 {
            "Payment interval must be between 1 and 30 days".to_string()
        } else {
            String::new()
        }
    });

    let amount_error = Signal::derive(move || {
        let a = amount.get();
        if a.is_empty() || usdt_to_raw(&a).is_some() {
            prepare_error.get()
        } else {
            "Enter a valid amount".to_string()
        }
    });

    let form_valid = Signal::derive(move || {
        let par = parcels_count.get();
        let inv = days_interval.get();
        usdt_to_raw(&amount.get()).is_some()
            && par >= 1 && par <= 12
            && inv >= 1 && inv <= 30
    });

    let submit_disabled = Signal::derive(move || !form_valid.get() || loading.get());

    let dispatch = {
        let amount = amount.clone();
        move |ev: SubmitEvent| {
            ev.prevent_default();
            let Some(raw) = usdt_to_raw(&amount.get()) else { return; };
            set_tx_status.set(LoanTxStatus::Idle);
            prepare.dispatch((
                raw,
                parcels_count.get_untracked(),
                days_interval.get_untracked(),
            ));
        }
    };

    view! {
        <Card hover=false tag="REQUEST LOAN">
            <form
                on:submit=dispatch
                style="display:flex; flex-direction:column; gap:var(--sp-6); margin-top:var(--sp-4)"
            >
                <AmountInput
                    label="Amount (USDT)"
                    hint="How much you need to borrow."
                    value=amount
                    set_value=set_amount
                    error=amount_error
                />

                <NumberInput
                    label="Number of installments"
                    value=parcels_count
                    set_value=set_parcels_count
                    hint="How many monthly payments (1–12)."
                />
                {move || {
                    let e = parcels_error.get();
                    if e.is_empty() { ().into_any() } else {
                        view! { <span class="form-hint" style="color:var(--c-red)">{e}</span> }.into_any()
                    }
                }}

                <NumberInput
                    label="Payment interval (days)"
                    value=days_interval
                    set_value=set_days_interval
                    hint="Days between each payment (1–30)."
                />
                {move || {
                    let e = interval_error.get();
                    if e.is_empty() { ().into_any() } else {
                        view! { <span class="form-hint" style="color:var(--c-red)">{e}</span> }.into_any()
                    }
                }}

                <Button
                    variant=BtnVariant::Primary
                    size=BtnSize::Lg
                    full_width=true
                    loading=loading
                    disabled=submit_disabled
                >
                    "REQUEST LOAN"
                </Button>
            </form>

            <LoanTxStatusCard
                tx_status=tx_status
                on_retry=Callback::new(move |()| {
                    set_tx_status.set(LoanTxStatus::Idle);
                    if let Some(raw) = usdt_to_raw(&amount.get()) {
                        prepare.dispatch((
                            raw,
                            parcels_count.get_untracked(),
                            days_interval.get_untracked(),
                        ));
                    }
                })
            />
        </Card>
    }
}

// ── STATUS CARD ────────────────────────────────────────────────

#[component]
fn LoanTxStatusCard(
    tx_status: ReadSignal<LoanTxStatus>,
    on_retry: Callback<()>,
) -> impl IntoView {
    move || match tx_status.get() {
        LoanTxStatus::Idle => ().into_any(),

        LoanTxStatus::Pending => view! {
            <div class="flex-center gap-4" style="padding: var(--sp-8) 0">
                <span class="spinner"></span>
                <span class="t-mono-sm t-muted">
                    "Waiting for signature and confirmation..."
                </span>
            </div>
        }
        .into_any(),

        LoanTxStatus::Complete(hash) => view! {
            <div class="flex-col gap-4 mt-4">
                <Alert kind=AlertKind::Success>
                    "Loan requisition submitted successfully!"
                </Alert>
                <div class="stat-block">
                    <span class="stat-label">"Transaction Hash"</span>
                    <HashDisplay value=hash />
                </div>
                <p class="t-mono-xs t-muted">
                    "Your request is now open for lenders to cover. \
                     Once fully covered, it will be funded automatically."
                </p>
            </div>
        }
        .into_any(),

        LoanTxStatus::Failed(err) => view! {
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
        }
        .into_any(),
    }
}
