use leptos::prelude::*;

use loan_machine_models::responses::{LoanRequisitionBundle, OpenMarketItem};
use crate::components::ui::*;
use crate::components::gas_modal::{use_gas_modal, GasEstimate, GasModalRequest};
use crate::wallet_auth::privy_bridge::{self, TxOutcome};
use crate::server_fns::loan_requisition::{get_open_market, prepare_cover_requisition};

#[component]
pub fn PendingRequisitions(#[prop(into)] coop_id: String) -> impl IntoView {
    let (refresh, set_refresh) = signal(0u32);
    let coop_id_res = coop_id.clone();

    let data = LocalResource::new(move || {
        let _ = refresh.get();
        let id = coop_id_res.clone();
        async move { get_open_market(id).await }
    });

    let on_covered = move || set_refresh.update(|n| *n += 1);

    view! {
        <Card hover=false tag="OPEN MARKET">
            <Suspense fallback=move || view! {
                <div class="flex-center" style="padding: var(--sp-8) 0">
                    <span class="spinner"></span>
                </div>
            }>
                {move || data.get().map(|res| match res {
                    Err(e) => view! {
                        <Alert kind=AlertKind::Error>{e.to_string()}</Alert>
                    }.into_any(),
                    Ok(market) if market.items.is_empty() => view! {
                        <p class="t-mono-sm t-muted"
                            style="padding: var(--sp-8) 0; text-align: center">
                            "No open loan requisitions in this cooperative."
                        </p>
                    }.into_any(),
                    Ok(market) => view! {
                        <div class="flex-col gap-4" style="margin-top: var(--sp-4)">
                            {market.items.into_iter().map(|item| {
                                let coop_id_row  = coop_id.clone();
                                let withdrawable = market.user_withdrawable.clone();
                                view! {
                                    <OpenMarketRow
                                        item=item
                                        coop_id=coop_id_row
                                        user_withdrawable=withdrawable
                                        on_covered=Callback::new(move |()| on_covered())
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

// ── ROW ─────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
enum CoverStatus { Idle, Pending, Done, Failed(String) }

#[component]
fn OpenMarketRow(
    item:              OpenMarketItem,
    #[prop(into)] coop_id: String,
    user_withdrawable: String,
    on_covered:        Callback<()>,
) -> impl IntoView {
    let req_id         = item.requisition_id.clone();
    let amount_display = fmt_usdt(&item.amount);
    let date_display   = fmt_date(item.created_at);
    let remaining_pct  = 100u32.saturating_sub(item.current_coverage);
    let max_pct        = max_affordable_pct(&user_withdrawable, &item.amount, remaining_pct);
    let initial_pct    = max_pct.max(1).min(remaining_pct);
    let (label, badge_color) = open_status_display(item.current_coverage);

    let (cover_pct, set_cover_pct) = signal(initial_pct);
    let (cover_status, set_cover_status) = signal(CoverStatus::Idle);

    let set_gas_modal = use_gas_modal();

    let prepare = Action::new({
        let coop_id = coop_id.clone();
        let req_id  = req_id.clone();
        move |pct: &u32| {
            let coop_id = coop_id.clone();
            let req_id  = req_id.clone();
            let pct     = *pct;
            async move { prepare_cover_requisition(coop_id, req_id, pct).await }
        }
    });

    let bundle: Signal<Option<LoanRequisitionBundle>> = Signal::derive(move || {
        match prepare.value().get() {
            Some(Ok(b)) => Some(b),
            _ => None,
        }
    });

    let loading = prepare.pending();

    let (submitted_pct, set_submitted_pct) = signal(initial_pct);

    // Open gas modal as soon as bundle is ready
    Effect::new(move |_| {
        if cover_status.get() != CoverStatus::Idle { return; }
        if loading.get() { return; }
        let Some(b) = bundle.get() else { return; };

        let pct = submitted_pct.get_untracked();
        let b   = b.clone();
        set_gas_modal.set(Some(GasModalRequest {
            title: "COVER LOAN REQUISITION".into(),
            estimates: vec![GasEstimate {
                label:   format!("Cover {pct}% of requisition"),
                gas_hex: b.gas_hex.clone(),
            }],
            on_confirm: Callback::new(move |()| {
                set_cover_status.set(CoverStatus::Pending);
                privy_bridge::send_tx(&b.loan_machine_address, &b.calldata, Some(&b.gas_hex));
            }),
        }));
    });

    privy_bridge::on_tx_outcome(move |outcome| {
        match (cover_status.get_untracked(), outcome) {
            (CoverStatus::Pending, TxOutcome::Complete(_)) => {
                set_cover_status.set(CoverStatus::Done);
                on_covered.run(());
            }
            (CoverStatus::Pending, TxOutcome::Failed(err)) => {
                set_cover_status.set(CoverStatus::Failed(err));
            }
            _ => {}
        }
    });

    // Unified error: server-side prepare failure OR wallet tx rejection
    let error_msg = Signal::derive(move || {
        if let Some(Err(e)) = prepare.value().get() {
            if !loading.get() { return e.to_string(); }
        }
        match cover_status.get() {
            CoverStatus::Failed(e) => e,
            _ => String::new(),
        }
    });

    let can_cover = max_pct > 0 && remaining_pct > 0;

    // Slider fill % relative to the slider's own range [1, max_pct]
    let slider_fill = Signal::derive(move || {
        if max_pct <= 1 { return 100u32; }
        cover_pct.get().saturating_sub(1) * 100 / (max_pct - 1)
    });

    view! {
        <div style="display:flex; flex-direction:column; gap: var(--sp-3); \
                    padding: var(--sp-5); border-radius: var(--radius-md); \
                    background: var(--c-surface-2); border: var(--border-thin)">

            // ── header ───────────────────────────────────────
            <div style="display:flex; justify-content:space-between; \
                        align-items:flex-start; gap: var(--sp-3)">
                <div style="display:flex; flex-direction:column; gap: 2px">
                    <span class="t-mono-xs t-muted">
                        "REQ #"{item.requisition_id.clone()}
                        " · "
                        {item.parcels_count}" parcels"
                        " · "
                        {date_display}
                    </span>
                    <span class="t-mono-lg t-yellow" style="font-weight:700">
                        {amount_display}
                    </span>
                    <span class="t-mono-xs" style="color: var(--c-gray-4)">
                        {fmt_borrower(&item.borrower)}
                    </span>
                </div>
                <span
                    class="t-mono-xs"
                    style=format!(
                        "padding: 3px 10px; border-radius: var(--radius-full); flex-shrink:0; \
                         background: {badge_color}22; color: {badge_color}; \
                         font-weight: 700; letter-spacing: 0.07em; white-space: nowrap"
                    )
                >
                    {label}
                </span>
            </div>

            // ── coverage bar ─────────────────────────────────
            <div style="display:flex; flex-direction:column; gap: 4px">
                <div style="display:flex; justify-content:space-between; align-items:center">
                    <span class="t-mono-xs t-muted">"Coverage"</span>
                    <span class="t-mono-sm t-bright" style="font-weight:600">
                        {item.current_coverage}
                        <span class="t-muted">"%  / 100%"</span>
                    </span>
                </div>
                <div style="background: var(--c-gray-3); border-radius: var(--radius-full); \
                            height: 8px; overflow: hidden">
                    <div style=format!(
                        "background: {}; width: {}%; height: 100%; \
                         transition: width 0.4s ease; border-radius: var(--radius-full)",
                        coverage_color(item.current_coverage),
                        item.current_coverage,
                    )></div>
                </div>
            </div>

            // ── cover section ─────────────────────────────────
            {if can_cover { view! {
                <div style="display:flex; flex-direction:column; gap: var(--sp-3); \
                            padding: var(--sp-4); background: var(--c-surface-3); \
                            border-radius: var(--radius-sm); border: var(--border-thin)">

                    // Large percentage display
                    <div style="display:flex; justify-content:space-between; align-items:center">
                        <span class="t-mono-xs t-muted">"Your contribution"</span>
                        <span style="font-family: var(--font-mono); font-size:1.6rem; \
                                     font-weight:700; color: var(--c-yellow); \
                                     line-height:1">
                            {move || format!("{}%", cover_pct.get())}
                        </span>
                    </div>

                    // Slider
                    <input
                        type="range"
                        class="cover-slider"
                        min="1"
                        max=max_pct.to_string()
                        prop:value=move || cover_pct.get().to_string()
                        style=move || format!(
                            "background: linear-gradient(to right, \
                             var(--c-yellow) {}%, var(--c-gray-3) {}%)",
                            slider_fill.get(), slider_fill.get()
                        )
                        on:input=move |ev| {
                            let v = event_target_value(&ev)
                                .parse::<u32>()
                                .unwrap_or(1)
                                .clamp(1, max_pct);
                            set_cover_pct.set(v);
                        }
                    />

                    // Range bounds labels
                    <div style="display:flex; justify-content:space-between">
                        <span class="t-mono-xs t-muted">"1%"</span>
                        <span class="t-mono-xs t-muted">
                            "max "{max_pct}"% of "{remaining_pct}"% remaining"
                        </span>
                    </div>

                    <Button
                        variant=BtnVariant::Secondary
                        size=BtnSize::Sm
                        loading=loading
                        disabled=Signal::derive(move || {
                            loading.get()
                                || cover_pct.get() == 0
                                || cover_status.get() == CoverStatus::Pending
                        })
                        on_click=Box::new(move || {
                            let pct = cover_pct.get_untracked();
                            set_submitted_pct.set(pct);
                            set_cover_status.set(CoverStatus::Idle);
                            prepare.dispatch(pct);
                        })
                    >
                        "COVER "
                        {move || format!("{}%", cover_pct.get())}
                    </Button>
                </div>
            }.into_any()} else { view! {
                <p class="t-mono-xs" style="color: var(--c-gray-4); font-style: italic">
                    {if remaining_pct == 0 {
                        "Fully covered — awaiting activation."
                    } else {
                        "You need donations to cover this requisition."
                    }}
                </p>
            }.into_any()}}

            // ── error ─────────────────────────────────────────
            {move || {
                let e = error_msg.get();
                if e.is_empty() { ().into_any() } else {
                    view! { <Alert kind=AlertKind::Error>{e}</Alert> }.into_any()
                }
            }}
        </div>
    }
}

// ── helpers ──────────────────────────────────────────────────

fn max_affordable_pct(withdrawable: &str, amount: &str, remaining: u32) -> u32 {
    let w: u128 = withdrawable.parse().unwrap_or(0);
    let a: u128 = amount.parse().unwrap_or(u128::MAX);
    if a == 0 { return 0; }
    let max = (w.saturating_mul(100) / a) as u32;
    max.min(remaining)
}

fn open_status_display(current_coverage: u32) -> (&'static str, &'static str) {
    if current_coverage == 0 {
        ("PENDING", "var(--c-yellow)")
    } else {
        ("PARTIAL", "var(--c-gold)")
    }
}

fn coverage_color(coverage: u32) -> &'static str {
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

fn fmt_borrower(addr: &str) -> String {
    if addr.len() >= 12 {
        format!("{}…{}", &addr[..8], &addr[addr.len()-4..])
    } else {
        addr.to_string()
    }
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
