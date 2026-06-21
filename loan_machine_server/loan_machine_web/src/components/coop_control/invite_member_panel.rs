use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::ui::*;
use crate::server_fns::admin_approvals::prepare_propose_wallet_as_admin;
use crate::wallet_auth::privy_bridge::{self, TxOutcome};

#[derive(Clone)]
enum InviteState {
    Idle,
    Preparing,
    Signing { target: String },
    Done { tx_hash: String, target: String },
    Error(String),
}

#[component]
pub fn InviteMemberPanel(#[prop(into)] coop_id: String) -> impl IntoView {
    let (wallet_input, set_wallet_input) = signal(String::new());
    let (invite_state, set_invite_state) = signal(InviteState::Idle);

    privy_bridge::on_tx_outcome(move |outcome| {
        let current = invite_state.get_untracked();
        let InviteState::Signing { target } = current else { return; };
        match outcome {
            TxOutcome::Complete(hash) => set_invite_state.set(InviteState::Done { tx_hash: hash, target }),
            TxOutcome::Failed(e)      => set_invite_state.set(InviteState::Error(e)),
        }
    });

    let on_submit = {
        let coop_id = coop_id.clone();
        move |_| {
            let target = wallet_input.get_untracked();
            if target.is_empty() { return; }
            let coop_id = coop_id.clone();
            set_invite_state.set(InviteState::Preparing);
            spawn_local(async move {
                match prepare_propose_wallet_as_admin(coop_id, target.clone()).await {
                    Ok(b) => {
                        set_invite_state.set(InviteState::Signing { target });
                        privy_bridge::send_tx(&b.to, &b.data, Some(&b.gas_hex));
                    }
                    Err(e) => set_invite_state.set(InviteState::Error(e.to_string())),
                }
            });
        }
    };

    view! {
        <Card variant=CardVariant::Default hover=false>
            <Badge color=BadgeColor::Gold>"INVITE MEMBER"</Badge>
            <h3 class="t-display-sm" style="margin-top: var(--sp-4); margin-bottom: var(--sp-6)">
                "Add a member to the cooperative"
            </h3>

            {move || match invite_state.get() {
                InviteState::Idle | InviteState::Preparing => {
                    let preparing = matches!(invite_state.get(), InviteState::Preparing);
                    view! {
                        <div class="flex-col gap-4">
                            <p class="t-mono-xs t-muted">
                                "Enter the wallet address of the person you want to invite. \
                                 This creates an approval proposal — once the threshold of \
                                 admins confirms it, the wallet can join."
                            </p>
                            <TextInput
                                label="Wallet address"
                                placeholder="0xabc123..."
                                hint="The Ethereum address of the new member"
                                value=wallet_input
                                set_value=set_wallet_input
                            />
                            <button
                                class="btn btn-gold"
                                disabled=move || preparing || wallet_input.get().is_empty()
                                on:click=on_submit.clone()
                            >
                                {if preparing {
                                    view! { <span class="spinner" style="width:1em;height:1em" /> }.into_any()
                                } else {
                                    view! { "PROPOSE APPROVAL" }.into_any()
                                }}
                            </button>
                        </div>
                    }.into_any()
                }

                InviteState::Signing { target } => view! {
                    <div class="flex-col gap-4">
                        <div class="stat-block">
                            <DataLabel>"Wallet"</DataLabel>
                            <HashDisplay value=target />
                        </div>
                        <div class="flex-center gap-4">
                            <span class="spinner"></span>
                            <span class="t-mono-sm t-muted">"Waiting for proposal signature..."</span>
                        </div>
                    </div>
                }.into_any(),

                InviteState::Done { tx_hash: hash, target } => view! {
                    <div class="flex-col gap-4">
                        <Alert kind=AlertKind::Success>"Approval proposal submitted!"</Alert>
                        <div class="stat-block">
                            <DataLabel>"Invited wallet"</DataLabel>
                            <HashDisplay value=target />
                        </div>
                        <div class="stat-block">
                            <DataLabel>"Transaction"</DataLabel>
                            <HashDisplay value=hash />
                        </div>
                        <p class="t-mono-xs t-muted">
                            "The wallet will be approved once the admin threshold confirms the proposal."
                        </p>
                        <button
                            class="btn btn-ghost"
                            on:click=move |_| {
                                set_wallet_input.set(String::new());
                                set_invite_state.set(InviteState::Idle);
                            }
                        >
                            "INVITE ANOTHER"
                        </button>
                    </div>
                }.into_any(),

                InviteState::Error(err) => view! {
                    <div class="flex-col gap-4">
                        <Alert kind=AlertKind::Error>{err}</Alert>
                        <button
                            class="btn btn-ghost"
                            on:click=move |_| set_invite_state.set(InviteState::Idle)
                        >
                            "TRY AGAIN"
                        </button>
                    </div>
                }.into_any(),
            }}
        </Card>
    }
}
