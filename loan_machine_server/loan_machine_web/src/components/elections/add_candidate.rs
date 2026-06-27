use leptos::prelude::*;

use loan_machine_models::wallet_address::WalletAddress;

use crate::components::ui::*;
use crate::components::gas_modal::{use_gas_modal, GasEstimate, GasModalRequest};
use crate::server_fns::elections::prepare_add_candidate;
use crate::wallet_auth::privy_bridge::{self, TxOutcome};

#[component]
pub fn AddCandidate(
    #[prop(into)] coop_id:     String,
                  election_id: u32,
    on_tx_success: Callback<()>,
) -> impl IntoView {
    let (input, set_input) = signal(String::new());
    let (input_err, set_input_err) = signal(String::new());
    let (error, set_error) = signal(String::new());

    let add_candidate = Action::new(
        move |(coop_id, eid, candidate): &(String, u32, WalletAddress)| {
            let coop_id   = coop_id.clone();
            let eid       = *eid;
            let candidate = *candidate;
            async move { prepare_add_candidate(coop_id, eid, candidate).await }
        }
    );
    let loading = add_candidate.pending();

    let gas_modal = use_gas_modal();
    Effect::new(move |_| {
        let Some(Ok(b)) = add_candidate.value().get() else { return };
        let to   = b.to.clone();
        let data = b.data.clone();
        let gas  = b.gas_hex.clone();
        gas_modal.set(Some(GasModalRequest {
            title: "CONFIRM ADD CANDIDATE".into(),
            estimates: vec![GasEstimate {
                label:   "Add candidate to election".into(),
                gas_hex: gas.clone(),
            }],
            on_confirm: Callback::new(move |_| {
                privy_bridge::send_tx(&to, &data, Some(&gas));
            }),
            on_cancel: None,
        }));
    });

    Effect::new(move |_| {
        if let Some(Err(e)) = add_candidate.value().get() {
            set_error.set(e.to_string());
        }
    });

    privy_bridge::on_tx_outcome(move |outcome| match outcome {
        TxOutcome::Complete(_) => {
            set_error.set(String::new());
            set_input.set(String::new());
            set_input_err.set(String::new());
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
                label="Add Candidate — Wallet Address"
                placeholder="0x0000...0000"
                hint="Must be a linked member who is not already a candidate"
                value=input
                set_value=set_input
                error=Signal::derive(move || input_err.get())
            />

            <Button
                variant=BtnVariant::Secondary
                full_width=true
                loading=Signal::derive(move || loading.get())
                on_click=Box::new(move || {
                    let raw = input.get_untracked();
                    match raw.trim().parse::<WalletAddress>() {
                        Err(_) => {
                            set_input_err.set("Invalid address (0x + 40 hex)".into());
                        }
                        Ok(candidate) => {
                            set_input_err.set(String::new());
                            set_error.set(String::new());
                            add_candidate.dispatch((coop_id.clone(), election_id, candidate));
                        }
                    }
                })
            >
                "ADD CANDIDATE"
            </Button>
        </div>
    }
}
