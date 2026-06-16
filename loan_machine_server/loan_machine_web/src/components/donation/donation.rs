// src/components/donation/donation.rs
//
// Reusable donation form: amount input + submit, then two sequential
// signatures via the Privy bridge — ERC20 `approve` followed by
// `LoanMachine.donate`. The confirm-and-sign step for each tx is
// delegated to the app-wide `<GasModal />` (mounted in `app.rs`), same
// as `FirstVinculationForm`.

use leptos::prelude::*;
use leptos::ev::SubmitEvent;

use crate::components::ui::*;
use crate::components::gas_modal::{use_gas_modal, GasEstimate, GasModalRequest};
use crate::wallet_auth::privy_bridge::{self, TxOutcome};
use crate::server_fns::donation::prepare_donation;
use loan_machine_models::responses::DonationBundle;

// ── STATE TYPES ───────────────────────────────────────────────

#[derive(Clone, PartialEq)]
enum DonationTxStatus {
    Idle,
    ApprovePending,
    AwaitingDonate,
    DonatePending,
    Complete(String), // donate tx hash
    Failed(String),   // error message
}

/// Open the "SIGN DONATION" gas modal for the `donate()` call. Shared by the
/// post-approve flow and the no-approve-needed (sufficient allowance) flow.
fn open_donate_modal(
    b: &DonationBundle,
    set_gas_modal: WriteSignal<Option<GasModalRequest>>,
    set_tx_status: WriteSignal<DonationTxStatus>,
) {
    let b = b.clone();
    set_gas_modal.set(Some(GasModalRequest {
        title: "SIGN DONATION".into(),
        estimates: vec![GasEstimate {
            label: "Donate to cooperative".into(),
            gas_hex: b.gas_donate.clone(),
        }],
        on_confirm: Callback::new(move |()| {
            set_tx_status.set(DonationTxStatus::DonatePending);
            privy_bridge::send_tx(&b.loan_machine_address, &b.donate_calldata, Some(&b.gas_donate));
        }),
    }));
}

// ── PUBLIC ENTRY POINT ───────────────────────────────────────

#[component]
pub fn DonationForm(
    #[prop(into)] coop_id: String,
    on_tx_success: impl Fn() + 'static,
) -> impl IntoView {
    let (amount, set_amount) = signal(String::new());

    let prepare = Action::new(move |raw_amount: &String| {
        let coop_id = coop_id.clone();
        let raw_amount = raw_amount.clone();
        async move { prepare_donation(coop_id, raw_amount).await }
    });

    let loading = prepare.pending();
    let prepare_error = Signal::derive(move || match prepare.value().get() {
        Some(Err(e)) => e.to_string(),
        _ => String::new(),
    });
    let bundle: Signal<Option<DonationBundle>> = Signal::derive(move || match prepare.value().get() {
        Some(Ok(b)) => Some(b),
        _ => None,
    });

    let (tx_status, set_tx_status) = signal(DonationTxStatus::Idle);
    let set_gas_modal = use_gas_modal();

    // Step 1: approve, as soon as the bundle is ready. If the donor already
    // has a sufficient USDT allowance, `prepare_donation` returns an empty
    // approve step ("0x") and we skip straight to the donate signature.
    Effect::new(move |_| {
        if tx_status.get() != DonationTxStatus::Idle {
            return;
        }
        if loading.get() { return; }
        let Some(b) = bundle.get() else { return; };

        if b.approve_calldata == "0x" {
            set_tx_status.set(DonationTxStatus::AwaitingDonate);
            open_donate_modal(&b, set_gas_modal, set_tx_status);
            return;
        }

        let b = b.clone();
        set_gas_modal.set(Some(GasModalRequest {
            title: "SIGN APPROVAL".into(),
            estimates: vec![GasEstimate {
                label: "Approve USDT spend".into(),
                gas_hex: b.gas_approve.clone(),
            }],
            on_confirm: Callback::new(move |()| {
                set_tx_status.set(DonationTxStatus::ApprovePending);
                privy_bridge::send_tx(&b.usdt_address, &b.approve_calldata, Some(&b.gas_approve));
            }),
        }));
    });
 
    // Step 2: as soon as the approval lands, immediately open the donate
    // signature request. Driven directly from the tx-outcome event rather
    // than a second watching effect, so "approve confirmed" and "prompt
    // for donate" happen in the same tick — no intermediate state for a
    // stray effect run to miss.
    privy_bridge::on_tx_outcome(move |outcome| {
        match (tx_status.get_untracked(), outcome) {
            (DonationTxStatus::ApprovePending, TxOutcome::Complete(_)) => {
                set_tx_status.set(DonationTxStatus::AwaitingDonate);

                let Some(b) = bundle.get_untracked() else { return; };
                open_donate_modal(&b, set_gas_modal, set_tx_status);
            }
            (DonationTxStatus::ApprovePending, TxOutcome::Failed(err)) => {
                set_tx_status.set(DonationTxStatus::Failed(err));
            }
            (DonationTxStatus::DonatePending, TxOutcome::Complete(hash)) => {
                set_tx_status.set(DonationTxStatus::Complete(hash));
            }
            (DonationTxStatus::DonatePending, TxOutcome::Failed(err)) => {
                set_tx_status.set(DonationTxStatus::Failed(err));
            }
            _ => {}
        }
    });

    Effect::new(move |_| {
        if matches!(tx_status.get(), DonationTxStatus::Complete(_)) {
            on_tx_success();
        }
    });

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let Some(raw) = usdt_to_raw(&amount.get()) else { return; };
        set_tx_status.set(DonationTxStatus::Idle);
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
        <Card hover=false tag="DONATE">
            <form
                on:submit=on_submit
                style="display:flex; flex-direction:column; gap:var(--sp-6); margin-top:var(--sp-4)"
            >
                <AmountInput
                    label="Amount (USDT)"
                    hint="Two signatures are required: approve, then donate."
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
                    "DONATE"
                </Button>
            </form>

            <DonationStatusCard
                tx_status=tx_status
                on_retry=Callback::new(move |()| {
                    set_tx_status.set(DonationTxStatus::Idle);
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
fn DonationStatusCard(
    tx_status: ReadSignal<DonationTxStatus>,
    on_retry: Callback<()>,
) -> impl IntoView {
    move || match tx_status.get() {
        DonationTxStatus::Idle => ().into_any(),

        DonationTxStatus::ApprovePending => view! {
            <div class="flex-center gap-4" style="padding: var(--sp-8) 0">
                <span class="spinner"></span>
                <span class="t-mono-sm t-muted">
                    "Waiting for approval signature and confirmation..."
                </span>
            </div>
        }
        .into_any(),

        DonationTxStatus::AwaitingDonate => view! {
            <div class="flex-center gap-4" style="padding: var(--sp-8) 0">
                <span class="spinner"></span>
                <span class="t-mono-sm t-muted">
                    "Approval confirmed. Review the donation signature..."
                </span>
            </div>
        }
        .into_any(),

        DonationTxStatus::DonatePending => view! {
            <div class="flex-center gap-4" style="padding: var(--sp-8) 0">
                <span class="spinner"></span>
                <span class="t-mono-sm t-muted">
                    "Waiting for donation signature and confirmation..."
                </span>
            </div>
        }
        .into_any(),

        DonationTxStatus::Complete(hash) => view! {
            <div class="flex-col gap-4 mt-4">
                <Alert kind=AlertKind::Success>
                    "Donation submitted successfully!"
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

        DonationTxStatus::Failed(err) => view! {
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
