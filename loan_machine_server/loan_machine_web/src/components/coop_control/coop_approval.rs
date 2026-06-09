// loan_machine_web/src/components/coop_approval/request_approval.rs

use leptos::prelude::*;

use crate::components::ui::*;
use crate::components::gas_modal::{use_gas_modal, GasEstimate, GasModalRequest};
use crate::server_fns::coop_approval::prepare_request_approval;
use crate::wallet_auth::privy_bridge::{self, TxOutcome};

#[component]
pub fn RequestApproval(
    #[prop(into)] coop_id: String,
    on_tx_success: Callback<()>,
) -> impl IntoView {
    let (error, set_error) = signal(String::new());

    let request = Action::new(move |coop_id: &String| {
        let id = coop_id.clone();
        async move { prepare_request_approval(id).await }
    });
    let loading = request.pending();

    let gas_modal = use_gas_modal();
    Effect::new(move |_| {
        let Some(Ok(b)) = request.value().get() else { return };
        let to   = b.to.clone();
        let data = b.data.clone();
        let gas  = b.gas_hex.clone();
        gas_modal.set(Some(GasModalRequest {
            title: "CONFIRM APPROVAL REQUEST".into(),
            estimates: vec![GasEstimate {
                label:   "Propose wallet approval".into(),
                gas_hex: gas.clone(),
            }],
            on_confirm: Callback::new(move |_| {
                privy_bridge::send_tx(&to, &data, Some(&gas));
            }),
        }));
    });

    Effect::new(move |_| {
        if let Some(Err(e)) = request.value().get() {
            set_error.set(e.to_string());
        }
    });

    privy_bridge::on_tx_outcome(move |outcome| match outcome {
        TxOutcome::Complete(_) => {
            set_error.set(String::new());
            on_tx_success.run(());
        }
        TxOutcome::Failed(err) => set_error.set(format!("Transaction failed: {err}")),
    });

    view! {
        <Card variant=CardVariant::Yellow tag="VISITOR — APPROVAL REQUEST" hover=false>
            <div class="flex-col gap-6" style="margin-top: var(--sp-4)">

                <p class="t-mono-sm">
                    "Your wallet does not yet belong to this cooperative. To join,
                    you must go through a collective approval process."
                </p>

                <div class="flex-col gap-3" style="padding: var(--sp-4) 0">
                    <span class="t-mono-xs t-muted">"HOW IT WORKS"</span>

                    <div style="display: flex; gap: var(--sp-3); align-items: flex-start">
                        <Badge color=BadgeColor::Gold>"1"</Badge>
                        <p class="t-mono-xs">
                            "You initiate an approval request here. This registers your
                            wallet as a candidate, but does not yet make it a member."
                        </p>
                    </div>

                    <div style="display: flex; gap: var(--sp-3); align-items: flex-start">
                        <Badge color=BadgeColor::Gold>"2"</Badge>
                        <p class="t-mono-xs">
                            "The cooperative's administrators review the request and register
                            their confirmations. The signature threshold defined by the
                            cooperative must be reached."
                        </p>
                    </div>

                    <div style="display: flex; gap: var(--sp-3); align-items: flex-start">
                        <Badge color=BadgeColor::Gold>"3"</Badge>
                        <p class="t-mono-xs">
                            "An elected moderator co-signs the request — this step ensures
                            the approval does not depend solely on admins."
                        </p>
                    </div>

                    <div style="display: flex; gap: var(--sp-3); align-items: flex-start">
                        <Badge color=BadgeColor::Gold>"4"</Badge>
                        <p class="t-mono-xs">
                            "With confirmations + co-signature, your wallet is approved.
                            You can then link your wallet to the cooperative and become a
                            full member."
                        </p>
                    </div>
                </div>

                <Alert kind=AlertKind::Info>
                    "After submitting, this screen will show the progress of the request
                    (admin confirmations, moderator co-signature) until approval."
                </Alert>

                {move || {
                    let e = error.get();
                    (!e.is_empty()).then(|| view! {
                        <Alert kind=AlertKind::Error>{e}</Alert>
                    })
                }}

                <Button
                    variant=BtnVariant::Primary
                    full_width=true
                    loading=Signal::derive(move || loading.get())
                    on_click=Box::new(move || {
                        set_error.set(String::new());
                        request.dispatch(coop_id.clone());
                    })
                >
                    "START APPROVAL REQUEST"
                </Button>
            </div>
        </Card>
    }
}