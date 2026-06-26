use leptos::prelude::*;

use loan_machine_models::responses::{ActiveLoanItem, LoanParcel, RepaymentBundle};
use crate::components::ui::*;
use crate::components::gas_modal::{use_gas_modal, GasEstimate, GasModalRequest};
use crate::wallet_auth::privy_bridge::{self, TxOutcome};
use crate::server_fns::loan_requisition::{get_my_active_loans, prepare_repayment};

// BorrowStatus discriminants (ILoanMachine.sol):
//   0=Pending, 1=PartiallyCovered, 2=FullyCovered, 3=Active,
//   4=Repaid, 5=Defaulted, 6=Cancelled

#[component]
pub fn MyPayments(#[prop(into)] coop_id: String) -> impl IntoView {
    let (refresh, set_refresh) = signal(0u32);
    let coop_id_res = coop_id.clone();

    let items = LocalResource::new(move || {
        let _ = refresh.get();
        let id = coop_id_res.clone();
        async move { get_my_active_loans(id).await }
    });

    let on_paid = move || set_refresh.update(|n| *n += 1);

    view! {
        <Card hover=false tag="MY PAYMENTS">
            <Suspense fallback=move || view! {
                <div class="flex-center" style="padding: var(--sp-8) 0">
                    <span class="spinner"></span>
                </div>
            }>
                {move || items.get().map(|res| match res {
                    Err(e) => view! {
                        <Alert kind=AlertKind::Error>{e.to_string()}</Alert>
                    }.into_any(),
                    Ok(list) if list.is_empty() => view! {
                        <p class="t-mono-sm t-muted"
                            style="padding: var(--sp-8) 0; text-align: center">
                            "No active loans with pending payments."
                        </p>
                    }.into_any(),
                    Ok(list) => view! {
                        <div class="flex-col gap-4" style="margin-top: var(--sp-4)">
                            {list.into_iter().map(|item| {
                                let coop_id_row = coop_id.clone();
                                view! {
                                    <LoanPaymentCard
                                        item=item
                                        coop_id=coop_id_row
                                        on_paid=Callback::new(move |()| on_paid())
                                    />
                                }
                            }).collect_view()}
                        </div>
                    }.into_any(),
                })}
            </Suspense>
        </Card>
    }
}

// ── PER-LOAN CARD ─────────────────────────────────────────────

#[derive(Clone, PartialEq)]
enum RepayStatus {
    Idle,
    ApprovePending,
    AwaitingRepay,
    RepayPending,
    Complete(String),
    Failed(String),
}

fn open_repay_modal(
    b: &RepaymentBundle,
    set_gas_modal: WriteSignal<Option<GasModalRequest>>,
    set_status:    WriteSignal<RepayStatus>,
) {
    let b = b.clone();
    set_gas_modal.set(Some(GasModalRequest {
        title: "SIGN REPAYMENT".into(),
        estimates: vec![GasEstimate {
            label:   "Repay loan parcel".into(),
            gas_hex: b.gas_repay.clone(),
        }],
        on_confirm: Callback::new(move |()| {
            set_status.set(RepayStatus::RepayPending);
            privy_bridge::send_tx(&b.loan_machine_address, &b.repay_calldata, Some(&b.gas_repay));
        }),
        on_cancel: Some(Callback::new(move |()| {
            set_status.set(RepayStatus::Failed("Transaction cancelled.".into()));
        })),
    }));
}

#[component]
fn LoanPaymentCard(
    item:     ActiveLoanItem,
    #[prop(into)] coop_id: String,
    on_paid:  Callback<()>,
) -> impl IntoView {
    let req_id        = item.requisition_id.clone();
    let amount_label  = fmt_usdt(&item.total_amount);
    let paid_count    = item.parcels_count.saturating_sub(item.parcels_pending);
    let next_amount   = fmt_usdt(&item.next_payment_amount);
    let can_pay       = item.can_pay;

    let set_gas_modal = use_gas_modal();
    let (repay_status, set_repay_status) = signal(RepayStatus::Idle);

    let prepare = Action::new({
        let coop_id = coop_id.clone();
        let req_id  = req_id.clone();
        move |()| {
            let coop_id = coop_id.clone();
            let req_id  = req_id.clone();
            async move { prepare_repayment(coop_id, req_id).await }
        }
    });

    let bundle: Signal<Option<RepaymentBundle>> = Signal::derive(move || {
        match prepare.value().get() {
            Some(Ok(b)) => Some(b),
            _ => None,
        }
    });

    let prepare_error = Signal::derive(move || match prepare.value().get() {
        Some(Err(e)) => e.to_string(),
        _ => String::new(),
    });

    let loading = prepare.pending();

    // Step 1 — approve (if needed), or skip straight to repay.
    Effect::new(move |_| {
        if repay_status.get() != RepayStatus::Idle { return; }
        if loading.get() { return; }
        let Some(b) = bundle.get() else { return; };

        if b.approve_calldata == "0x" {
            set_repay_status.set(RepayStatus::AwaitingRepay);
            open_repay_modal(&b, set_gas_modal, set_repay_status);
            return;
        }

        let b2 = b.clone();
        set_gas_modal.set(Some(GasModalRequest {
            title: "SIGN APPROVAL".into(),
            estimates: vec![GasEstimate {
                label:   "Approve USDT spend".into(),
                gas_hex: b2.gas_approve.clone(),
            }],
            on_confirm: Callback::new(move |()| {
                set_repay_status.set(RepayStatus::ApprovePending);
                privy_bridge::send_tx(&b2.usdt_address, &b2.approve_calldata, Some(&b2.gas_approve));
            }),
            on_cancel: Some(Callback::new(move |()| {
                set_repay_status.set(RepayStatus::Failed("Transaction cancelled.".into()));
            })),
        }));
    });

    // Step 2 — after approval lands, prompt for the repay signature.
    privy_bridge::on_tx_outcome(move |outcome| {
        match (repay_status.get_untracked(), outcome) {
            (RepayStatus::ApprovePending, TxOutcome::Complete(_)) => {
                set_repay_status.set(RepayStatus::AwaitingRepay);
                let Some(b) = bundle.get_untracked() else { return; };
                open_repay_modal(&b, set_gas_modal, set_repay_status);
            }
            (RepayStatus::ApprovePending, TxOutcome::Failed(err)) => {
                set_repay_status.set(RepayStatus::Failed(err));
            }
            (RepayStatus::RepayPending, TxOutcome::Complete(hash)) => {
                set_repay_status.set(RepayStatus::Complete(hash));
                on_paid.run(());
            }
            (RepayStatus::RepayPending, TxOutcome::Failed(err)) => {
                set_repay_status.set(RepayStatus::Failed(err));
            }
            _ => {}
        }
    });

    let parcels = item.parcels.clone();

    view! {
        <div style="display:flex; flex-direction:column; gap: var(--sp-3); \
                    padding: var(--sp-4); border-radius: var(--radius-md); \
                    background: var(--c-surface-2); border: var(--border-thin)">

            // ── header ───────────────────────────────────────
            <div style="display:flex; justify-content:space-between; align-items:flex-start; gap: var(--sp-3)">
                <div style="display:flex; flex-direction:column; gap: 2px">
                    <span class="t-mono-xs t-muted">
                        "REQ #"{item.requisition_id.clone()}
                        " · "
                        {paid_count}"/"{item.parcels_count}" parcels paid"
                    </span>
                    <span class="t-mono-lg t-yellow" style="font-weight:700">
                        {amount_label}
                    </span>
                </div>
                <span
                    class="t-mono-xs"
                    style="padding: 3px 10px; border-radius: var(--radius-full); flex-shrink:0; \
                           background: var(--c-green)22; color: var(--c-green); \
                           font-weight: 700; letter-spacing: 0.07em"
                >
                    "ACTIVE"
                </span>
            </div>

            // ── progress bar ─────────────────────────────────
            {
                let pct = if item.parcels_count > 0 {
                    (paid_count * 100 / item.parcels_count).min(100)
                } else { 0 };
                view! {
                    <div>
                        <div style="display:flex; justify-content:space-between; margin-bottom: 4px">
                            <span class="t-mono-xs t-muted">"Repayment progress"</span>
                            <span class="t-mono-sm t-bright" style="font-weight:600">
                                {paid_count}
                                <span class="t-muted">" / "{item.parcels_count}</span>
                            </span>
                        </div>
                        <div style="background: var(--c-gray-3); border-radius: var(--radius-full); \
                                    height: 8px; overflow: hidden">
                            <div style=format!(
                                "background: var(--c-green); width: {}%; height: 100%; \
                                 transition: width 0.4s ease; border-radius: var(--radius-full)",
                                pct,
                            )></div>
                        </div>
                    </div>
                }
            }

            // ── parcel table ─────────────────────────────────
            <div style="display:flex; flex-direction:column; gap: 2px; margin-top: var(--sp-2)">
                {parcels.into_iter().map(|p| view! {
                    <ParcelRow parcel=p />
                }).collect_view()}
            </div>

            // ── pay button ────────────────────────────────────
            {if can_pay {
                let next_amount2 = next_amount.clone();
                view! {
                    <div style="display:flex; align-items:center; justify-content:space-between; \
                                padding-top: var(--sp-2); gap: var(--sp-3)">
                        <span class="t-mono-sm t-muted">
                            "Next payment: "
                            <span class="t-yellow" style="font-weight:600">{next_amount2}</span>
                        </span>
                        <Button
                            variant=BtnVariant::Primary
                            size=BtnSize::Sm
                            loading=loading
                            disabled=Signal::derive(move || {
                                loading.get()
                                    || matches!(repay_status.get(),
                                        RepayStatus::ApprovePending
                                        | RepayStatus::AwaitingRepay
                                        | RepayStatus::RepayPending)
                            })
                            on_click=Box::new(move || {
                                set_repay_status.set(RepayStatus::Idle);
                                prepare.dispatch(());
                            })
                        >
                            "PAY"
                        </Button>
                    </div>
                }.into_any()
            } else {
                // Show next due amount even when not yet payable, for visibility.
                view! {
                    <div style="padding-top: var(--sp-2)">
                        <span class="t-mono-xs t-muted">
                            "Next parcel: "
                            <span style="font-weight:600">{next_amount}</span>
                            " — not due yet"
                        </span>
                    </div>
                }.into_any()
            }}

            // ── prepare error ─────────────────────────────────
            {move || {
                let err = prepare_error.get();
                if err.is_empty() { ().into_any() } else {
                    view! { <Alert kind=AlertKind::Error>{err}</Alert> }.into_any()
                }
            }}

            // ── tx status ─────────────────────────────────────
            {move || match repay_status.get() {
                RepayStatus::ApprovePending => view! {
                    <div class="flex-center gap-4" style="padding-top: var(--sp-2)">
                        <span class="spinner"></span>
                        <span class="t-mono-xs t-muted">"Waiting for USDT approval..."</span>
                    </div>
                }.into_any(),
                RepayStatus::AwaitingRepay => view! {
                    <div class="flex-center gap-4" style="padding-top: var(--sp-2)">
                        <span class="spinner"></span>
                        <span class="t-mono-xs t-muted">"Approval confirmed. Review repayment signature..."</span>
                    </div>
                }.into_any(),
                RepayStatus::RepayPending => view! {
                    <div class="flex-center gap-4" style="padding-top: var(--sp-2)">
                        <span class="spinner"></span>
                        <span class="t-mono-xs t-muted">"Submitting repayment..."</span>
                    </div>
                }.into_any(),
                RepayStatus::Complete(hash) => view! {
                    <div class="flex-col gap-2" style="padding-top: var(--sp-2)">
                        <Alert kind=AlertKind::Success>"Parcel paid successfully!"</Alert>
                        <div class="stat-block">
                            <span class="stat-label">"Transaction Hash"</span>
                            <HashDisplay value=hash />
                        </div>
                    </div>
                }.into_any(),
                RepayStatus::Failed(err) => view! {
                    <Alert kind=AlertKind::Error>{err}</Alert>
                }.into_any(),
                RepayStatus::Idle => view! { <span></span> }.into_any(),
            }}
        </div>
    }
}

// ── PARCEL ROW ────────────────────────────────────────────────

#[component]
fn ParcelRow(parcel: LoanParcel) -> impl IntoView {
    let (label, bg, fg) = if parcel.is_paid {
        ("PAID",     "var(--c-green)22",  "var(--c-green)")
    } else {
        ("PENDING",  "var(--c-yellow)22", "var(--c-yellow)")
    };

    view! {
        <div style="display:flex; align-items:center; justify-content:space-between; \
                    padding: 6px 8px; border-radius: var(--radius-sm); \
                    background: var(--c-surface-3)">
            <span class="t-mono-xs t-muted" style="min-width: 60px">
                "#"{parcel.index + 1}
            </span>
            <span class="t-mono-xs t-muted" style="flex:1">
                {fmt_date(parcel.due_date)}
            </span>
            <span class="t-mono-xs t-bright" style="font-weight:600; min-width: 130px; text-align:right">
                {fmt_usdt(&parcel.amount)}
            </span>
            <span
                class="t-mono-xs"
                style=format!(
                    "margin-left: 10px; padding: 2px 8px; border-radius: var(--radius-full); \
                     background: {bg}; color: {fg}; font-weight:700; letter-spacing:0.05em; \
                     min-width: 60px; text-align:center"
                )
            >
                {label}
            </span>
        </div>
    }
}

// ── helpers ───────────────────────────────────────────────────

fn fmt_usdt(raw: &str) -> String {
    let n: u128 = raw.parse().unwrap_or(0);
    let whole    = n / 1_000_000;
    let frac     = n % 1_000_000;
    if frac == 0 { format!("{whole} USDT") }
    else          { format!("{whole}.{frac:06} USDT") }
}

fn fmt_date(secs: u64) -> String {
    if secs == 0 { return "—".into(); }
    let (y, m, d) = civil_date((secs / 86_400) as i64);
    format!("{y}-{m:02}-{d:02}")
}

fn civil_date(days: i64) -> (i64, i64, i64) {
    let z   = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y   = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp  = (5 * doy + 2) / 153;
    let d   = doy - (153 * mp + 2) / 5 + 1;
    let m   = if mp < 10 { mp + 3 } else { mp - 9 };
    let y   = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}
