// src/components/donation/withdrawal.rs
//
// Reusable withdrawal form: amount input + submit, then a single
// signature via the Privy bridge — `LoanMachine.withdraw`. The
// confirm-and-sign step is delegated to the app-wide `<GasModal />`
// (mounted in `app.rs`), same as `DonationForm`.

use leptos::prelude::*;
use leptos::ev::SubmitEvent;

use crate::components::ui::*;
use crate::components::gas_modal::{use_gas_modal, GasEstimate, GasModalRequest};
use crate::wallet_auth::privy_bridge::{self, TxOutcome};
use crate::server_fns::withdrawal::prepare_withdrawal;
use loan_machine_models::responses::WithdrawalBundle;

// ── STATE TYPES ───────────────────────────────────────────────

#[derive(Clone, PartialEq)]
enum WithdrawalTxStatus {
    Idle,
    Pending,
    Complete(String), // withdraw tx hash
    Failed(String),   // error message
}

// ── PUBLIC ENTRY POINT ───────────────────────────────────────

#[component]
pub fn WithdrawalForm(
    #[prop(into)] coop_id: String,
    on_tx_success: impl Fn() + 'static,
) -> impl IntoView {
    let (amount, set_amount) = signal(String::new());

    let prepare = Action::new(move |raw_amount: &String| {
        let coop_id = coop_id.clone();
        let raw_amount = raw_amount.clone();
        async move { prepare_withdrawal(coop_id, raw_amount).await }
    });

    let loading = prepare.pending();
    let prepare_error = Signal::derive(move || match prepare.value().get() {
        Some(Err(e)) => e.to_string(),
        _ => String::new(),
    });
    let bundle: Signal<Option<WithdrawalBundle>> = Signal::derive(move || match prepare.value().get() {
        Some(Ok(b)) => Some(b),
        _ => None,
    });

    let (tx_status, set_tx_status) = signal(WithdrawalTxStatus::Idle);
    let set_gas_modal = use_gas_modal();

    // As soon as the bundle is ready, open the withdraw signature request.
    Effect::new(move |_| {
        if tx_status.get() != WithdrawalTxStatus::Idle {
            return;
        }
        if loading.get() { return; }
        let Some(b) = bundle.get() else { return; };

        let b = b.clone();
        set_gas_modal.set(Some(GasModalRequest {
            title: "SIGN WITHDRAWAL".into(),
            estimates: vec![GasEstimate {
                label: "Withdraw from cooperative".into(),
                gas_hex: b.gas_withdraw.clone(),
            }],
            on_confirm: Callback::new(move |()| {
                set_tx_status.set(WithdrawalTxStatus::Pending);
                privy_bridge::send_tx(&b.loan_machine_address, &b.withdraw_calldata, Some(&b.gas_withdraw));
            }),
            on_cancel: Some(Callback::new(move |()| {
                set_tx_status.set(WithdrawalTxStatus::Failed("Transaction cancelled.".into()));
            })),
        }));
    });

    privy_bridge::on_tx_outcome(move |outcome| {
        match (tx_status.get_untracked(), outcome) {
            (WithdrawalTxStatus::Pending, TxOutcome::Complete(hash)) => {
                set_tx_status.set(WithdrawalTxStatus::Complete(hash));
            }
            (WithdrawalTxStatus::Pending, TxOutcome::Failed(err)) => {
                set_tx_status.set(WithdrawalTxStatus::Failed(err));
            }
            _ => {}
        }
    });

    Effect::new(move |_| {
        if matches!(tx_status.get(), WithdrawalTxStatus::Complete(_)) {
            on_tx_success();
        }
    });

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let Some(raw) = usdt_to_raw(&amount.get()) else { return; };
        set_tx_status.set(WithdrawalTxStatus::Idle);
        prepare.dispatch(raw);
    };

    let amount_error = Signal::derive(move || {
        let a = amount.get();
        if a.is_empty() || usdt_to_raw(&a).is_some() {
            prepare_error.get()
        } else {
            "Enter a valid amount".to_string()
        }
    });

    let submit_disabled = Signal::derive(move || {
        usdt_to_raw(&amount.get()).is_none() || loading.get()
    });

    view! {
        <Card hover=false tag="WITHDRAW">
            <form
                on:submit=on_submit
                style="display:flex; flex-direction:column; gap:var(--sp-6); margin-top:var(--sp-4)"
            >
                <AmountInput
                    label="Amount (USDT)"
                    hint="A single signature is required to withdraw."
                    value=amount
                    set_value=set_amount
                    error=amount_error
                />

                <Button
                    variant=BtnVariant::Primary
                    size=BtnSize::Lg
                    full_width=true
                    loading=loading
                    disabled=submit_disabled
                >
                    "WITHDRAW"
                </Button>
            </form>

            <WithdrawalStatusCard
                tx_status=tx_status
                on_retry=Callback::new(move |()| {
                    set_tx_status.set(WithdrawalTxStatus::Idle);
                    if let Some(raw) = usdt_to_raw(&amount.get()) {
                        prepare.dispatch(raw);
                    }
                })
            />
        </Card>
    }
}

// ── STATUS CARD ────────────────────────────────────────────────

#[component]
fn WithdrawalStatusCard(
    tx_status: ReadSignal<WithdrawalTxStatus>,
    on_retry: Callback<()>,
) -> impl IntoView {
    move || match tx_status.get() {
        WithdrawalTxStatus::Idle => ().into_any(),

        WithdrawalTxStatus::Pending => view! {
            <div class="flex-center gap-4" style="padding: var(--sp-8) 0">
                <span class="spinner"></span>
                <span class="t-mono-sm t-muted">
                    "Waiting for withdrawal signature and confirmation..."
                </span>
            </div>
        }
        .into_any(),

        WithdrawalTxStatus::Complete(hash) => view! {
            <div class="flex-col gap-4 mt-4">
                <Alert kind=AlertKind::Success>
                    "Withdrawal submitted successfully!"
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
        }
        .into_any(),

        WithdrawalTxStatus::Failed(err) => view! {
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
