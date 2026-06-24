use leptos::prelude::*;

use loan_machine_models::responses::{LoanRequisitionBundle, LoanRequisitionItem};
use crate::components::ui::*;
use crate::components::gas_modal::{use_gas_modal, GasEstimate, GasModalRequest};
use crate::wallet_auth::privy_bridge::{self, TxOutcome};
use crate::server_fns::loan_requisition::{get_my_requisitions, prepare_cancel_requisition};

// BorrowStatus discriminants (ILoanMachine.sol):
//   0=Pending, 1=PartiallyCovered, 2=FullyCovered, 3=Active,
//   4=Repaid, 5=Defaulted, 6=Cancelled

#[component]
pub fn MyRequisitions(#[prop(into)] coop_id: String) -> impl IntoView {
    let (refresh, set_refresh) = signal(0u32);
    let coop_id_res = coop_id.clone();

    let items = LocalResource::new(move || {
        let _ = refresh.get();
        let id = coop_id_res.clone();
        async move { get_my_requisitions(id).await }
    });

    let on_cancelled = move || set_refresh.update(|n| *n += 1);

    view! {
        <Card hover=false tag="MY LOAN REQUISITIONS">
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
                            "You have no loan requisitions yet."
                        </p>
                    }.into_any(),
                    Ok(list) => view! {
                        <div class="flex-col gap-4" style="margin-top: var(--sp-4)">
                            {list.into_iter().map(|item| {
                                let coop_id_row = coop_id.clone();
                                view! {
                                    <RequisitionRow
                                        item=item
                                        coop_id=coop_id_row
                                        on_cancelled=Callback::new(move |()| on_cancelled())
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

// ── ROW ──────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
enum CancelStatus { Idle, Pending, Done, Failed(String) }

#[component]
fn RequisitionRow(
    item:         LoanRequisitionItem,
    #[prop(into)] coop_id: String,
    on_cancelled: Callback<()>,
) -> impl IntoView {
    let req_id         = item.requisition_id.clone();
    let amount_display = fmt_usdt(&item.amount);
    let date_display   = fmt_date(item.created_at);
    let coverage       = item.current_coverage;
    let cancellable    = is_cancellable(item.status, coverage);
    let (label, color) = status_display(item.status);

    let set_gas_modal = use_gas_modal();
    let (cancel_status, set_cancel_status) = signal(CancelStatus::Idle);

    let prepare = Action::new({
        let coop_id = coop_id.clone();
        let req_id  = req_id.clone();
        move |()| {
            let coop_id = coop_id.clone();
            let req_id  = req_id.clone();
            async move { prepare_cancel_requisition(coop_id, req_id).await }
        }
    });

    let bundle: Signal<Option<LoanRequisitionBundle>> = Signal::derive(move || {
        match prepare.value().get() {
            Some(Ok(b)) => Some(b),
            _ => None,
        }
    });

    let loading = prepare.pending();

    // Open gas modal as soon as the bundle is ready.
    Effect::new(move |_| {
        if cancel_status.get() != CancelStatus::Idle { return; }
        if loading.get() { return; }
        let Some(b) = bundle.get() else { return; };

        let b = b.clone();
        set_gas_modal.set(Some(GasModalRequest {
            title: "CANCEL LOAN REQUEST".into(),
            estimates: vec![GasEstimate {
                label:   "Cancel loan requisition".into(),
                gas_hex: b.gas_hex.clone(),
            }],
            on_confirm: Callback::new(move |()| {
                set_cancel_status.set(CancelStatus::Pending);
                privy_bridge::send_tx(&b.loan_machine_address, &b.calldata, Some(&b.gas_hex));
            }),
            on_cancel: Some(Callback::new(move |()| {
                set_cancel_status.set(CancelStatus::Failed("Transaction cancelled.".into()));
            })),
        }));
    });

    privy_bridge::on_tx_outcome(move |outcome| {
        match (cancel_status.get_untracked(), outcome) {
            (CancelStatus::Pending, TxOutcome::Complete(_)) => {
                set_cancel_status.set(CancelStatus::Done);
                on_cancelled.run(());
            }
            (CancelStatus::Pending, TxOutcome::Failed(err)) => {
                set_cancel_status.set(CancelStatus::Failed(err));
            }
            _ => {}
        }
    });

    let cancel_error = Signal::derive(move || match cancel_status.get() {
        CancelStatus::Failed(e) => e,
        _ => String::new(),
    });

    view! {
        <div style="display:flex; flex-direction:column; gap: var(--sp-2); \
                    padding: var(--sp-4); border-radius: var(--radius-md); \
                    background: var(--c-surface-2); border: var(--border-thin)">

            // ── header row ──────────────────────────────────
            <div style="display:flex; justify-content:space-between; align-items:flex-start; gap: var(--sp-3)">
                <div style="display:flex; flex-direction:column; gap: 2px">
                    <span class="t-mono-xs t-muted">
                        "REQ #"{item.requisition_id}
                        " · "
                        {item.parcels_count}" parcels"
                        " · "
                        {date_display}
                    </span>
                    <span class="t-mono-lg t-yellow" style="font-weight:700">
                        {amount_display}
                    </span>
                </div>
                <span
                    class="t-mono-xs"
                    style=format!(
                        "padding: 3px 10px; border-radius: var(--radius-full); flex-shrink:0; \
                         background: {color}22; color: {color}; \
                         font-weight: 700; letter-spacing: 0.07em; white-space: nowrap"
                    )
                >
                    {label}
                </span>
            </div>

            // ── coverage bar ─────────────────────────────────
            <div>
                <div style="display:flex; justify-content:space-between; margin-bottom: 4px">
                    <span class="t-mono-xs t-muted">"Coverage"</span>
                    <span class="t-mono-sm t-bright" style="font-weight:600">
                        {coverage}
                        <span class="t-muted">"%  / 100%"</span>
                    </span>
                </div>
                <div style="background: var(--c-gray-3); border-radius: var(--radius-full); \
                            height: 8px; overflow: hidden">
                    <div style=format!(
                        "background: {}; width: {}%; height: 100%; \
                         transition: width 0.4s ease; border-radius: var(--radius-full)",
                        coverage_bar_color(coverage, item.status),
                        coverage,
                    )></div>
                </div>
            </div>

            // ── cancel button ─────────────────────────────────
            {if cancellable { view! {
                <div style="display:flex; justify-content:flex-end">
                    <Button
                        variant=BtnVariant::Ghost
                        size=BtnSize::Sm
                        loading=loading
                        disabled=Signal::derive(move || loading.get() || cancel_status.get() == CancelStatus::Pending)
                        on_click=Box::new(move || {
                            set_cancel_status.set(CancelStatus::Idle);
                            prepare.dispatch(());
                        })
                    >
                        "CANCEL"
                    </Button>
                </div>
            }.into_any()} else { ().into_any() }}

            // ── cancel error ──────────────────────────────────
            {move || {
                let e = cancel_error.get();
                if e.is_empty() { ().into_any() } else {
                    view! {
                        <Alert kind=AlertKind::Error>{e}</Alert>
                    }.into_any()
                }
            }}
        </div>
    }
}

// ── helpers ───────────────────────────────────────────────────

fn is_cancellable(status: u8, current_coverage: u32) -> bool {
    // Active(3) and Cancelled(6) cannot be cancelled; neither can fully covered.
    status != 3 && status != 6 && current_coverage < 100
}

fn status_display(status: u8) -> (&'static str, &'static str) {
    match status {
        0 => ("PENDING",   "var(--c-yellow)"),
        1 => ("PARTIAL",   "var(--c-gold)"),
        2 => ("COVERED",   "var(--c-green)"),
        3 => ("ACTIVE",    "var(--c-green)"),
        4 => ("REPAID",    "var(--c-gray-4)"),
        5 => ("DEFAULTED", "var(--c-red)"),
        6 => ("CANCELLED", "var(--c-gray-4)"),
        _ => ("UNKNOWN",   "var(--c-gray-4)"),
    }
}

fn coverage_bar_color(coverage: u32, status: u8) -> &'static str {
    if status == 5 { return "var(--c-red)"; }
    if coverage >= 100 { "var(--c-green)" }
    else if coverage > 0 { "var(--c-gold)" }
    else { "var(--c-gray-3)" }
}

fn fmt_usdt(raw: &str) -> String {
    let n: u128 = raw.parse().unwrap_or(0);
    let whole   = n / 1_000_000;
    let frac    = n % 1_000_000;
    if frac == 0 { format!("{whole} USDT") }
    else         { format!("{whole}.{frac:06} USDT") }
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
