// Member financial status panel.
//
// Accepts a coop_id, fetches the member's on-chain/subgraph financial state,
// and renders it as a read-only card.  Used in:
//   • CoopRow (user_coops.rs) — summary row for each coop the user is a member of
//   • RoleSection (coop_control_panel.rs) — inside the member/admin/moderator arm

use leptos::prelude::*;

use loan_machine_models::responses::MemberFinancials;
use crate::components::ui::*;
use crate::server_fns::member_financials::get_member_financials;

#[component]
pub fn MemberStatusPanel(#[prop(into)] coop_id: String) -> impl IntoView {
    let data = LocalResource::new(move || {
        let id = coop_id.clone();
        async move { get_member_financials(id).await }
    });

    view! {
        <div class="member-status-panel">
            <Suspense fallback=move || view! {
                <div class="flex-center" style="padding: var(--sp-8) 0">
                    <span class="spinner"></span>
                </div>
            }>
                {move || data.get().map(|res| match res {
                    Err(e) => view! {
                        <Alert kind=AlertKind::Error>{e.to_string()}</Alert>
                    }.into_any(),
                    Ok(f) => view! { <FinancialsCard f=f /> }.into_any(),
                })}
            </Suspense>
        </div>
    }
}

#[component]
fn FinancialsCard(f: MemberFinancials) -> impl IntoView {
    let has_debt        = f.borrowing != "0";
    let has_donation    = f.donation  != "0";

    view! {
        <Card hover=false>
            <span class="card-tag">"FINANCIAL STATUS"</span>
            <div class="flex-col gap-6" style="margin-top: var(--sp-6)">

                // Member identity
                <div class="form-group">
                    <label class="form-label">"Member ID"</label>
                    <HashDisplay value=f.member_id.clone() />
                </div>

                // Reputation
                <div style="display: flex; gap: var(--sp-3); align-items: center; flex-wrap: wrap">
                    <StatBlock label="Reputation" value=f.reputation.to_string() />
                    {(f.reputation > 0).then(|| view! {
                        <Badge color=BadgeColor::Green filled=true>"ACTIVE"</Badge>
                    })}
                    {(f.reputation == 0).then(|| view! {
                        <Badge color=BadgeColor::Gold>"NO HISTORY"</Badge>
                    })}
                    {(f.reputation < 0).then(|| view! {
                        <Badge color=BadgeColor::Red>"DEFAULTED"</Badge>
                    })}
                </div>

                // Donation summary
                <div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: var(--sp-4)">
                    <StatBlock
                        label="Donation"
                        value=fmt_usdt(&f.donation)
                    />
                    <StatBlock
                        label="In coverage"
                        value=fmt_usdt(&f.in_coverage)
                    />
                    <StatBlock
                        label="Available for withdrawal"
                        value=fmt_usdt(&f.withdrawable)
                    />
                </div>

                // Borrow summary
                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: var(--sp-4)">
                    <StatBlock
                        label="Active loans"
                        value=fmt_usdt(&f.borrowing)
                    />
                    <StatBlock
                        label="Last loan"
                        value=fmt_timestamp(f.last_borrow_time)
                    />
                </div>

                // Badges / quick status row
                <div style="display: flex; gap: var(--sp-2); flex-wrap: wrap">
                    {has_donation.then(|| view! {
                        <Badge color=BadgeColor::Green filled=true>"DONOR"</Badge>
                    })}
                    {has_debt.then(|| view! {
                        <Badge color=BadgeColor::Red filled=true>"DEBTOR"</Badge>
                    })}
                    {(!has_donation && !has_debt).then(|| view! {
                        <Badge color=BadgeColor::Gold>"NO ACTIVITY"</Badge>
                    })}
                </div>
            </div>
        </Card>
    }
}

// ── formatting helpers ────────────────────────────────────────

/// Format a raw USDT amount (6 decimals) as "1 234.567890 USDT".
fn fmt_usdt(raw: &str) -> String {
    let n: u128 = raw.parse().unwrap_or(0);
    let whole   = n / 1_000_000;
    let frac    = n % 1_000_000;
    if frac == 0 {
        format!("{whole} USDT")
    } else {
        format!("{whole}.{frac:06} USDT")
    }
}

/// Format a Unix timestamp as a short date, or "Never" for 0.
fn fmt_timestamp(ts: u64) -> String {
    if ts == 0 { return "Never".into(); }
    let (y, m, d) = civil_date((ts / 86_400) as i64);
    format!("{y}-{m:02}-{d:02}")
}

/// Convert days-since-Unix-epoch to (year, month, day) using the
/// Gregorian calendar. Algorithm by Howard Hinnant (public domain).
fn civil_date(z: i64) -> (i32, u32, u32) {
    let z   = z + 719_468;
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp  = (5 * doy + 2) / 153;
    let d   = doy - (153 * mp + 2) / 5 + 1;
    let m   = if mp < 10 { mp + 3 } else { mp - 9 };
    let y   = yoe as i64 + era * 400 + if m <= 2 { 1 } else { 0 };
    (y as i32, m as u32, d as u32)
}
