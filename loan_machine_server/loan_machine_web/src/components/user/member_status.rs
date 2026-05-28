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
    let has_allowance   = f.allowance != "0";

    view! {
        <Card hover=false>
            <span class="card-tag">"STATUS FINANCEIRO"</span>
            <div class="flex-col gap-6" style="margin-top: var(--sp-6)">

                // Member identity
                <div class="form-group">
                    <label class="form-label">"Member ID"</label>
                    <HashDisplay value=f.member_id.clone() />
                </div>

                // Reputation
                <div style="display: flex; gap: var(--sp-3); align-items: center; flex-wrap: wrap">
                    <StatBlock label="Reputação" value=f.reputation.to_string() />
                    {(f.reputation > 0).then(|| view! {
                        <Badge color=BadgeColor::Green filled=true>"ATIVO"</Badge>
                    })}
                    {(f.reputation == 0).then(|| view! {
                        <Badge color=BadgeColor::Gold>"SEM HISTÓRICO"</Badge>
                    })}
                    {(f.reputation < 0).then(|| view! {
                        <Badge color=BadgeColor::Red>"INADIMPLENTE"</Badge>
                    })}
                </div>

                // Donation summary
                <div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: var(--sp-4)">
                    <StatBlock
                        label="Doação"
                        value=fmt_usdt(&f.donation)
                    />
                    <StatBlock
                        label="Em cobertura"
                        value=fmt_usdt(&f.in_coverage)
                    />
                    <StatBlock
                        label="Disponível p/ saque"
                        value=fmt_usdt(&f.withdrawable)
                    />
                </div>

                // Borrow summary
                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: var(--sp-4)">
                    <StatBlock
                        label="Empréstimos ativos"
                        value=fmt_usdt(&f.borrowing)
                    />
                    <StatBlock
                        label="Último empréstimo"
                        value=fmt_timestamp(f.last_borrow_time)
                    />
                </div>

                // Allowance row — only shown when non-zero
                {has_allowance.then(|| view! {
                    <div class="form-group">
                        <label class="form-label">"Aprovação USDT ao contrato"</label>
                        <p class="t-mono-xs t-muted">{fmt_usdt(&f.allowance)}</p>
                    </div>
                })}

                // Badges / quick status row
                <div style="display: flex; gap: var(--sp-2); flex-wrap: wrap">
                    {has_donation.then(|| view! {
                        <Badge color=BadgeColor::Green filled=true>"DOADOR"</Badge>
                    })}
                    {has_debt.then(|| view! {
                        <Badge color=BadgeColor::Red filled=true>"DEVEDOR"</Badge>
                    })}
                    {(!has_donation && !has_debt).then(|| view! {
                        <Badge color=BadgeColor::Gold>"SEM MOVIMENTAÇÃO"</Badge>
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

/// Format a Unix timestamp as a short date, or "Nunca" for 0.
fn fmt_timestamp(ts: u64) -> String {
    if ts == 0 { return "Nunca".into(); }
    let days             = ts / 86_400;
    let epoch_day_offset = 719_162u64;
    let day              = days + epoch_day_offset;
    let year             = day / 365;
    let month            = (day % 365) / 30 + 1;
    let dom              = (day % 365) % 30 + 1;
    format!("{year}-{month:02}-{dom:02}")
}
