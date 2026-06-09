// loan_machine_web/src/components/elections/vote_election.rs

use leptos::prelude::*;

use loan_machine_models::wallet_address::WalletAddress;

use crate::components::ui::*;
use crate::components::gas_modal::{use_gas_modal, GasEstimate, GasModalRequest};
use crate::server_fns::elections::prepare_vote;
use crate::wallet_auth::privy_bridge::{self, TxOutcome};

#[component]
pub fn VoteElection(
    #[prop(into)] coop_id:     String,
                  election_id: u32,
    on_tx_success: Callback<()>,
) -> impl IntoView {
    let (input, set_input) = signal(String::new());
    let (error, set_error) = signal(String::new());

    let cast_vote = Action::new(
        move |(coop_id, eid, cand): &(String, u32, WalletAddress)| {
            let coop_id = coop_id.clone();
            let eid     = *eid;
            let cand    = *cand;
            async move { prepare_vote(coop_id, eid, cand).await }
        }
    );
    let loading = cast_vote.pending();

    let gas_modal = use_gas_modal();
    Effect::new(move |_| {
        let Some(Ok(b)) = cast_vote.value().get() else { return };
        let to   = b.to.clone();
        let data = b.data.clone();
        let gas  = b.gas_hex.clone();
        gas_modal.set(Some(GasModalRequest {
            title: "CONFIRM VOTE".into(),
            estimates: vec![GasEstimate {
                label:   "Vote for candidate".into(),
                gas_hex: gas.clone(),
            }],
            on_confirm: Callback::new(move |_| {
                privy_bridge::send_tx(&to, &data, Some(&gas));
            }),
        }));
    });

    Effect::new(move |_| {
        if let Some(Err(e)) = cast_vote.value().get() {
            set_error.set(e.to_string());
        }
    });

    privy_bridge::on_tx_outcome(move |outcome| match outcome {
        TxOutcome::Complete(_) => {
            set_error.set(String::new());
            set_input.set(String::new());
            on_tx_success.run(());
        }
        TxOutcome::Failed(err) => {
            set_error.set(format!("Transaction failed: {err}"));
        }
    });

    view! {
        <div class="form-group">
            {move || {
                let e = error.get();
                (!e.is_empty()).then(|| view! {
                    <Alert kind=AlertKind::Error>{e}</Alert>
                })
            }}

            <TextInput
                label="Your vote — Paste the candidate address"
                placeholder="0x0000...0000"
                hint="Copy the address above and paste here"
                value=input
                set_value=set_input
                error=Signal::derive(move || String::new())
            />

            <Button
                variant=BtnVariant::Primary
                full_width=true
                loading=Signal::derive(move || loading.get())
                on_click=Box::new(move || {
                    let raw = input.get_untracked();
                    match raw.trim().parse::<WalletAddress>() {
                        Err(_) => set_error.set("Invalid address (0x + 40 hex)".into()),
                        Ok(cand) => {
                            set_error.set(String::new());
                            cast_vote.dispatch((coop_id.clone(), election_id, cand));
                        }
                    }
                })
            >
                "CONFIRM VOTE"
            </Button>
        </div>
    }
}