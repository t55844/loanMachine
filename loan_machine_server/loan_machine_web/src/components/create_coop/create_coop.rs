// loan_machine_web/src/components/create_coop.rs
//
// Six-step flow that chains two server calls and two on-chain txs.
// See create_coop_steps.rs for the per-step rendering.

use leptos::prelude::*;
use loan_machine_models::{
    requests::{CreateCoopRequest, RegisterDeployedCoopRequest},
    responses::CoopRegistrationResult,
};
use loan_machine_models::wallet_address::WalletAddress;

use crate::components::ui::*;
use crate::components::create_coop::create_coop_steps::{
    DoneStep, FormStep, SignInitStep, StepProgress, WaitingStep,
};
use crate::server_fns::create_coop::{prepare_create_coop, register_deployed_coop};
use crate::components::gas_modal::{use_gas_modal, GasEstimate, GasModalRequest};
use crate::wallet_auth::privy_bridge::{self, DeployOutcome, TxOutcome};

// ── Step state machine ──────────────────────────────────────

#[derive(Clone, PartialEq)]
pub enum CoopStep {
    Form,
    WaitingDeploy,
    SignInit,
    Registering,
    Done,
}

impl CoopStep {
    fn index(&self) -> usize {
        match self {
            CoopStep::Form          => 0,
            CoopStep::WaitingDeploy => 1,
            CoopStep::SignInit      => 2,
            CoopStep::Registering   => 3,
            CoopStep::Done          => 4,
        }
    }
}

// ── CreateCoopPage ──────────────────────────────────────────

#[component]
pub fn CreateCoopPage(#[prop(into)] founder_wallet: WalletAddress) -> impl IntoView {
    // ── Step + form-field signals ───────────────────────────
    let (step, set_step) = signal(CoopStep::Form);

    let (name,   set_name)   = signal(String::new());
    let (admin2, set_admin2) = signal(String::new());

    let (name_err,   set_name_err)   = signal(String::new());
    let (admin2_err, set_admin2_err) = signal(String::new());

    // Single top-of-page error display.
    let (error, set_error) = signal(String::new());

    // Results from the two on-chain steps.
    let (contract_address, set_contract_address) = signal(String::new());
    let (reg_result, set_reg_result) = signal(Option::<CoopRegistrationResult>::None);

    // ── Server-fn Actions ───────────────────────────────────
    let prepare = Action::new(|req: &CreateCoopRequest| {
        let req = req.clone();
        async move { prepare_create_coop(req).await }
    });

    let register = Action::new(move |_: &()| {
        let req = RegisterDeployedCoopRequest {
            name:                 name.get_untracked(),
            loan_machine_address: contract_address.get_untracked(),
            founder_wallet,
        };
        async move { register_deployed_coop(req).await }
    });

    // ── Derived signals ─────────────────────────────────────
    let loading = prepare.pending();
    let bundle = Signal::derive(move || match prepare.value().get() {
        Some(Ok(b)) => Some(b),
        _ => None,
    });

    // ── Step transitions ────────────────────────────────────
    let advance = move |next: CoopStep| {
        if next.index() > step.get_untracked().index() {
            set_error.set(String::new());
        }
        set_step.set(next);
    };

    let gas_modal = use_gas_modal();

    // ── Prepare action settles — open deploy modal immediately ──
    Effect::new(move |_| {
        let Some(Ok(b)) = prepare.value().get() else {
            if let Some(Err(e)) = prepare.value().get() { set_error.set(e.to_string()); }
            return;
        };
        if step.get_untracked() != CoopStep::Form { return; }

        let dd = b.deploy_data.clone();
        let gd = b.gas_deploy.clone();
        gas_modal.set(Some(GasModalRequest {
            title: "CONFIRM DEPLOY".into(),
            estimates: vec![GasEstimate {
                label:   "LoanMachine Deploy".into(),
                gas_hex: gd.clone(),
            }],
            on_confirm: Callback::new(move |_| {
                advance(CoopStep::WaitingDeploy);
                privy_bridge::deploy_contract(&dd, &gd);
            }),
        }));
    });

    // ── TX 1 outcome (deploy) ───────────────────────────────
    privy_bridge::on_deploy_outcome(move |outcome| match outcome {
        DeployOutcome::Complete { contract_address: addr, .. } => {
            set_contract_address.set(addr);
            advance(CoopStep::SignInit);
        }
        DeployOutcome::Failed(err) => {
            set_error.set(format!("Deploy failed: {err}"));
            advance(CoopStep::Form);
        }
    });

    // ── TX 2 outcome (init multisig) ────────────────────────
    privy_bridge::on_tx_outcome(move |outcome| match outcome {
        TxOutcome::Complete(_hash) => {
            advance(CoopStep::Registering);
            register.dispatch(());
        }
        TxOutcome::Failed(err) => {
            set_error.set(format!("Initialization failed: {err}"));
        }
    });

    // ── Register action settles ─────────────────────────────
    Effect::new(move |_| {
        let Some(result) = register.value().get() else { return };
        match result {
            Ok(r) => {
                set_reg_result.set(Some(r));
                if step.get_untracked() == CoopStep::Registering {
                    advance(CoopStep::Done);
                }
            }
            Err(e) => {
                set_error.set(e.to_string());
                if step.get_untracked() == CoopStep::Registering {
                    advance(CoopStep::Form);
                }
            }
        }
    });

    // ── Step dispatch ───────────────────────────────────────
    view! {
        <Section>
            <div class="container-sm">

                <div class="t-center" style="margin-bottom: var(--sp-8)">
                    <SectionTitle>"CREATE COOPERATIVE"</SectionTitle>
                    <StepProgress step=step />
                </div>

                {move || {
                    let e = error.get();
                    (!e.is_empty()).then(|| view! {
                        <div style="margin-bottom: var(--sp-6)">
                            <Alert kind=AlertKind::Error>{e}</Alert>
                        </div>
                    })
                }}

                {move || match step.get() {

                    CoopStep::Form => view! {
                        <FormStep
                            name set_name
                            admin2 set_admin2
                            name_err admin2_err
                            loading
                            founder_wallet=founder_wallet
                            on_submit=Box::new(move || {
                                let n  = name.get_untracked();
                                let a2 = admin2.get_untracked();

                                let a2_parsed: Result<WalletAddress, _> = a2.parse();

                                let name_ok   = !n.trim().is_empty();
                                let admin2_ok = a2_parsed.is_ok();

                                set_name_err.set(if name_ok {
                                    String::new()
                                } else {
                                    "Name is required".into()
                                });
                                set_admin2_err.set(if admin2_ok {
                                    String::new()
                                } else {
                                    "Invalid address (0x + 40 hex)".into()
                                });

                                if let (true, Ok(a2_addr)) = (name_ok, a2_parsed) {
                                    prepare.dispatch(CreateCoopRequest {
                                        name:          n,
                                        founder_wallet,
                                        admin_wallets: vec![founder_wallet, a2_addr],
                                        threshold:     2,
                                    });
                                }
                            })
                        />
                    }.into_any(),

                    CoopStep::WaitingDeploy => view! {
                        <WaitingStep
                            label="Waiting for deploy confirmation…"
                            hint="Sign the transaction in your Privy wallet"
                        />
                    }.into_any(),

                    CoopStep::SignInit => {
                        let addr      = contract_address.get();
                        let init_data = bundle.get().map(|b| b.initialize_data.clone()).unwrap_or_default();
                        let gas_init  = bundle.get().map(|b| b.gas_initialize.clone()).unwrap_or_default();
                        let addr_tx   = addr.clone();

                        let gas_modal = use_gas_modal();

                        view! {
                            <SignInitStep
                                contract_address=addr
                                on_sign=Box::new(move || {
                                    let a = addr_tx.clone();
                                    let d = init_data.clone();
                                    let g = gas_init.clone();
                                    gas_modal.set(Some(GasModalRequest {
                                        title: "CONFIRM INITIALIZATION".into(),
                                        estimates: vec![GasEstimate {
                                            label:   "Initialize Multisig".into(),
                                            gas_hex: g.clone(),
                                        }],
                                        on_confirm: Callback::new(move |_| {
                                            privy_bridge::send_tx(&a, &d, Some(&g));
                                        }),
                                    }));
                                })
                            />
                        }.into_any()
                    },

                    CoopStep::Registering => view! {
                        <WaitingStep
                            label="Registering cooperative on the blockchain…"
                            hint="The server is signing the registration transaction"
                        />
                    }.into_any(),

                    CoopStep::Done => view! {
                        <DoneStep reg_result />
                    }.into_any(),
                }}

            </div>
        </Section>
    }
}