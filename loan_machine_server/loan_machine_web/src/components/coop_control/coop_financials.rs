// Cooperative-wide money flow and status panel.
//
// Shows aggregate figures (total donations, outstanding loans, available
// balance, contract balance) plus active-member count and average
// reputation. Shown on the coop control panel for every viewer, regardless
// of role — distinct from `MemberStatusPanel`, which is per-member.

use leptos::prelude::*;

use loan_machine_models::responses::CoopFinancials;
use crate::components::ui::*;
use crate::server_fns::coop_financials::get_coop_financials;

#[component]
pub fn CoopFinancialsPanel(#[prop(into)] coop_id: String) -> impl IntoView {
    let data = LocalResource::new(move || {
        let id = coop_id.clone();
        async move { get_coop_financials(id).await }
    });

    view! {
        <div class="coop-financials-panel">
            <Suspense fallback=move || view! {
                <div class="flex-center" style="padding: var(--sp-8) 0">
                    <span class="spinner"></span>
                </div>
            }>
                {move || data.get().map(|res| match res {
                    Err(e) => view! {
                        <Alert kind=AlertKind::Error>{e.to_string()}</Alert>
                    }.into_any(),
                    Ok(f) => view! { <CoopFinancialsCard f=f /> }.into_any(),
                })}
            </Suspense>
        </div>
    }
}

#[component]
fn CoopFinancialsCard(f: CoopFinancials) -> impl IntoView {
    let (badge_color, badge_text) = if f.is_active {
        (BadgeColor::Green, "ACTIVE")
    } else {
        (BadgeColor::Red, "INACTIVE")
    };

    view! {
        <Card hover=false tag="MONEY FLOW">
            <div class="flex-col gap-6" style="margin-top: var(--sp-6)">

                <div style="display: flex; gap: var(--sp-3); align-items: center; flex-wrap: wrap">
                    <Badge color=badge_color filled=true live=f.is_active>{badge_text}</Badge>
                    <StatBlock label="Active members" value=f.active_member_count.to_string() />
                    <StatBlock label="Average reputation" value=f.average_reputation.to_string() />
                </div>

                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: var(--sp-4)">
                    <StatBlock
                        label="Total donations"
                        value=fmt_usdt(&f.total_donations)
                    />
                    <StatBlock
                        label="Outstanding loans"
                        value=fmt_usdt(&f.total_borrowed)
                    />
                    <StatBlock
                        label="Available balance"
                        value=fmt_usdt(&f.available_balance)
                    />
                    <StatBlock
                        label="Contract balance"
                        value=fmt_usdt(&f.contract_balance)
                    />
                </div>
            </div>
        </Card>
    }
}

/// Format a raw USDT amount (6 decimals) as "1234.567890 USDT".
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
