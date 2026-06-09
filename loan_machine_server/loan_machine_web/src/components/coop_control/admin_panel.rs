use leptos::prelude::*;
use leptos::task::spawn_local;

use loan_machine_models::responses::{ApproveWalletPending, ApproveWalletProposalRow, RequestApprovalBundle};

use crate::components::ui::*;
use crate::server_fns::admin_approvals::{
    list_pending_approvals, prepare_confirm_proposal, prepare_cosign_proposal,
};
use crate::wallet_auth::privy_bridge;

#[derive(Clone, Copy, PartialEq)]
pub enum ApprovalAction { AdminConfirm, ModeratorCosign }

// ── pure logic, exposed for tests ─────────────────────────────────────────

pub(crate) fn viewer_has_acted(row: &ApproveWalletProposalRow, action: ApprovalAction) -> bool {
    match action {
        ApprovalAction::AdminConfirm    => row.viewer_confirmed,
        ApprovalAction::ModeratorCosign => row.moderator_cosigned,
    }
}

pub(crate) fn action_button_label(action: ApprovalAction, is_sending: bool) -> &'static str {
    match (action, is_sending) {
        (_,                              true)  => "Sending…",
        (ApprovalAction::AdminConfirm,    false) => "Confirm",
        (ApprovalAction::ModeratorCosign, false) => "Co-sign",
    }
}


#[component]
pub fn AdminPanel(coop_id: String, on_tx_success: Callback<()>) -> impl IntoView {
    view! {
        <Card variant=CardVariant::Default hover=false>
            <Badge color=BadgeColor::Gold>"ADMIN PANEL"</Badge>
            <h3 class="t-display-sm" style="margin-top: var(--sp-4); margin-bottom: var(--sp-6)">
                "Pending wallet approvals"
            </h3>
            <ApprovalsList
                coop_id=coop_id
                action=ApprovalAction::AdminConfirm
                on_tx_success=on_tx_success
            />
        </Card>
    }
}

#[component]
pub fn ModeratorPanel(coop_id: String, on_tx_success: Callback<()>) -> impl IntoView {
    view! {
        <Card variant=CardVariant::Default hover=false>
            <Badge color=BadgeColor::Yellow>"MODERATOR PANEL"</Badge>
            <h3 class="t-display-sm" style="margin-top: var(--sp-4); margin-bottom: var(--sp-6)">
                "Pending approval co-signatures"
            </h3>
            <ApprovalsList
                coop_id=coop_id
                action=ApprovalAction::ModeratorCosign
                on_tx_success=on_tx_success
            />
        </Card>
    }
}

#[component]
fn ApprovalsList(
    coop_id: String,
    action: ApprovalAction,
    on_tx_success: Callback<()>,
) -> impl IntoView {
    let (refresh,    set_refresh)    = signal(0u64);
    let (active_id,  set_active_id)  = signal(Option::<u64>::None);
    let (error,      set_error)      = signal(Option::<String>::None);

    let coop_id_for_fetch = coop_id.clone();
    let pending = LocalResource::new(move || {
        let id = coop_id_for_fetch.clone();
        let _  = refresh.get();
        async move { list_pending_approvals(id).await }
    });

    // Subscribe once. The handler ignores outcomes that don't match an
    // in-flight tx (covers the case where some other component on the
    // page also broadcasts privy_tx_complete).
    privy_bridge::on_tx_outcome(move |outcome| {
        if active_id.get_untracked().is_none() { return; }
        set_active_id.set(None);
        match outcome {
            privy_bridge::TxOutcome::Complete(_) => {
                set_error.set(None);
                set_refresh.update(|n| *n += 1);
                on_tx_success.run(());
            }
            privy_bridge::TxOutcome::Failed(e) => set_error.set(Some(e)),
        }
    });

    // submit: row asks us to fire a tx. Returns nothing; outcome arrives async.
    let submit = Callback::new(move |proposal_id: u64| {
        let coop_id = coop_id.clone();
        set_error.set(None);
        set_active_id.set(Some(proposal_id));
        spawn_local(async move {
            let prepared: Result<RequestApprovalBundle, _> = match action {
                ApprovalAction::AdminConfirm    => prepare_confirm_proposal(coop_id, proposal_id).await,
                ApprovalAction::ModeratorCosign => prepare_cosign_proposal(coop_id, proposal_id).await,
            };
            match prepared {
                Ok(b)  => privy_bridge::send_tx(&b.to, &b.data, Some(&b.gas_hex)),
                Err(e) => {
                    set_active_id.set(None);
                    set_error.set(Some(e.to_string()));
                }
            }
        });
    });

    view! {
        <Suspense fallback=|| view! { <span class="spinner" /> }>
            {move || pending.get().map(|res| match res {
                Err(e) => view! { <Alert kind=AlertKind::Error>{e.to_string()}</Alert> }.into_any(),
                Ok(p) if p.proposals.is_empty() => view! {
                    <p class="t-mono-sm t-muted">"No pending items."</p>
                }.into_any(),
                Ok(p) => render_list(p, action, submit, active_id, error),
            })}
        </Suspense>
    }
}

fn render_list(
    p: ApproveWalletPending,
    action: ApprovalAction,
    submit: Callback<u64>,
    active_id: ReadSignal<Option<u64>>,
    error: ReadSignal<Option<String>>,
) -> AnyView {
    let threshold    = p.threshold;
    let total_admins = p.total_admins;
    view! {
        <p class="t-mono-xs t-muted" style="margin-bottom: var(--sp-4)">
            {format!("Threshold: {threshold}/{total_admins} admins")}
        </p>
        {move || error.get().map(|e| view! {
            <div style="margin-bottom: var(--sp-4)">
                <Alert kind=AlertKind::Error>{e}</Alert>
            </div>
        })}
        <div class="flex-col gap-4">
            {p.proposals.into_iter().map(|row| view! {
                <ProposalRow
                    row=row
                    threshold=threshold
                    action=action
                    submit=submit
                    active_id=active_id
                />
            }).collect_view()}
        </div>
    }.into_any()
}

#[component]
fn ProposalRow(
    row: ApproveWalletProposalRow,
    threshold: u32,
    action: ApprovalAction,
    submit: Callback<u64>,
    active_id: ReadSignal<Option<u64>>,
) -> impl IntoView {
    let proposal_id   = row.proposal_id;
    let proposer      = row.proposer.clone();
    let created_at    = row.created_at;
    let confirmations = row.confirmations;
    let mod_cosigned  = row.moderator_cosigned;
    let viewer_acted = viewer_has_acted(&row, action);

    let is_mine_pending = Memo::new(move |_| active_id.get() == Some(proposal_id));
    let any_pending     = Memo::new(move |_| active_id.get().is_some());

    

    view! {
        <div class="stat-block">
            <div class="flex-between mb-2">
                <DataLabel>{format!("Proposal #{}", proposal_id)}</DataLabel>
                <span class="t-mono-xs t-muted">{format!("ts {created_at}")}</span>
            </div>

            <div class="form-group">
                <label class="form-label">"Wallet to approve"</label>
                <HashDisplay value=proposer />
            </div>

            <div style="display: flex; gap: var(--sp-3); flex-wrap: wrap; margin-top: var(--sp-3)">
                <Badge color=BadgeColor::Yellow>
                    {format!("{confirmations}/{threshold} confirmations")}
                </Badge>
                {if mod_cosigned {
                    view! { <Badge color=BadgeColor::Green filled=true>"CO-SIGNED"</Badge> }.into_any()
                } else {
                    view! { <Badge color=BadgeColor::Red>"AWAITING CO-SIGN."</Badge> }.into_any()
                }}
                {viewer_acted.then(|| view! { <Badge color=BadgeColor::Green>"YOU ALREADY ACTED"</Badge> })}
            </div>

            {move || (!viewer_acted).then(|| view! {
                <button
                    class="btn btn-gold"
                    style="margin-top: var(--sp-4)"
                    disabled=move || any_pending.get()
                    on:click=move |_| submit.run(proposal_id)
                >
                    {move || if is_mine_pending.get() { action_button_label(action, true) }
                            else                      { action_button_label(action, false) }}
                </button>
            })}
        </div>
    }
}