// loan_machine_web/src/components/create_coop.rs
//
// Six-step flow that chains two server calls and two on-chain txs.
// See create_coop_steps.rs for the per-step rendering.
//
// CHANGES vs prior version:
//   • The single `cpf_cnpj` String signal is replaced by the pair
//     (doc_kind, document) — same shape vinculation uses.
//   • `CreateCoopRequest` now carries `doc_kind: DocKind` and
//     `document: String` instead of a single `member_id: String`.
//     See loan_machine_models/src/requests.rs — you'll need to update
//     that struct accordingly.  Note at the bottom of this turn lists
//     every downstream change.

use leptos::prelude::*;
use loan_machine_models::{
    requests::{CreateCoopRequest, DocKind, RegisterDeployedCoopRequest},
    responses::CoopRegistrationResult,
};
use loan_machine_models::wallet_address::WalletAddress;

use crate::components::ui::*;
use crate::components::create_coop::create_coop_steps::{
    AccessCodeStep, DoneStep, FormStep, SignInitStep, StepProgress, WaitingStep,
};
use crate::server_fns::create_coop::{prepare_create_coop, register_deployed_coop};
use crate::components::gas_modal::{use_gas_modal, GasEstimate, GasModalRequest};
use crate::wallet_auth::privy_bridge::{self, DeployOutcome, TxOutcome};

// ── Step state machine ──────────────────────────────────────

#[derive(Clone, PartialEq)]
pub enum CoopStep {
    Form,
    AccessCode,
    WaitingDeploy,
    SignInit,
    Registering,
    Done,
}

impl CoopStep {
    fn index(&self) -> usize {
        match self {
            CoopStep::Form          => 0,
            CoopStep::AccessCode    => 1,
            CoopStep::WaitingDeploy => 2,
            CoopStep::SignInit      => 3,
            CoopStep::Registering   => 4,
            CoopStep::Done          => 5,
        }
    }
}

// ── CreateCoopPage ──────────────────────────────────────────

#[component]
pub fn CreateCoopPage(#[prop(into)] founder_wallet: WalletAddress) -> impl IntoView {
    // ── Step + form-field signals ───────────────────────────
    let (step, set_step) = signal(CoopStep::Form);

    let (name,     set_name)     = signal(String::new());
    let (doc_kind, set_doc_kind) = signal(DocKind::Cnpj);
    let (document, set_document) = signal(String::new());
    let (admin2,   set_admin2)   = signal(String::new());
    let (admin3,   set_admin3)   = signal(String::new());

    let (name_err,     set_name_err)     = signal(String::new());
    let (document_err, set_document_err) = signal(String::new());
    let (admin2_err,   set_admin2_err)   = signal(String::new());
    let (admin3_err,   set_admin3_err)   = signal(String::new());

    // Single top-of-page error display.
    let (error, set_error) = signal(String::new());

    // Results from the two on-chain steps.
    let (contract_address, set_contract_address) = signal(String::new());
    let (reg_result, set_reg_result) = signal(Option::<CoopRegistrationResult>::None);

    let (code_saved, set_code_saved) = signal(false);

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

    // ── Prepare action settles ──────────────────────────────
    Effect::new(move |_| {
        let Some(result) = prepare.value().get() else { return };
        match result {
            Ok(_) => {
                if step.get_untracked() == CoopStep::Form {
                    advance(CoopStep::AccessCode);
                }
            }
            Err(e) => set_error.set(e.to_string()),
        }
    });

    // ── TX 1 outcome (deploy) ───────────────────────────────
    privy_bridge::on_deploy_outcome(move |outcome| match outcome {
        DeployOutcome::Complete { contract_address: addr, .. } => {
            set_contract_address.set(addr);
            advance(CoopStep::SignInit);
        }
        DeployOutcome::Failed(err) => {
            set_error.set(format!("Deploy failed: {err}"));
            advance(CoopStep::AccessCode);
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
                            doc_kind set_doc_kind
                            document set_document
                            admin2 set_admin2
                            admin3 set_admin3
                            name_err document_err admin2_err admin3_err
                            loading
                            founder_wallet=founder_wallet
                            on_submit=Box::new(move || {
                                let n        = name.get_untracked();
                                let dk       = doc_kind.get_untracked();
                                let doc      = document.get_untracked();
                                let a2       = admin2.get_untracked();
                                let a3       = admin3.get_untracked();

                                let a2_parsed: Result<WalletAddress, _> = a2.parse();
                                let a3_parsed: Result<WalletAddress, _> = a3.parse();

                                let name_ok     = !n.trim().is_empty();
                                let document_ok = !doc.trim().is_empty()
                                    && dk.bare_char_count(&doc) == dk.max_digits();
                                let admin2_ok   = a2_parsed.is_ok();
                                let admin3_ok   = a3_parsed.is_ok();

                                set_name_err.set(if name_ok {
                                    String::new()
                                } else {
                                    "Name is required".into()
                                });
                                set_document_err.set(if document_ok {
                                    String::new()
                                } else {
                                    match dk {
                                        DocKind::Cpf  => "CPF must have 11 digits".into(),
                                        DocKind::Cnpj => "CNPJ must have 14 characters".into(),
                                    }
                                });
                                set_admin2_err.set(if admin2_ok {
                                    String::new()
                                } else {
                                    "Invalid address (0x + 40 hex)".into()
                                });
                                set_admin3_err.set(if admin3_ok {
                                    String::new()
                                } else {
                                    "Invalid address (0x + 40 hex)".into()
                                });

                                if let (true, true, Ok(a2_addr), Ok(a3_addr))
                                    = (name_ok, document_ok, a2_parsed, a3_parsed)
                                {
                                    prepare.dispatch(CreateCoopRequest {
                                        name:          n,
                                        founder_wallet,
                                        admin_wallets: vec![founder_wallet, a2_addr, a3_addr],
                                        threshold:     2,
                                        doc_kind:      dk,
                                        document:      doc,
                                    });
                                }
                            })
                        />
                    }.into_any(),

                    CoopStep::AccessCode => {
                        let access_code = bundle.get().map(|b| b.access_code.clone()).unwrap_or_default();
                        let deploy_data = bundle.get().map(|b| b.deploy_data.clone()).unwrap_or_default();
                        let gas_deploy  = bundle.get().map(|b| b.gas_deploy.clone()).unwrap_or_default();

                        let gas_modal = use_gas_modal();

                        view! {
                            <AccessCodeStep
                                access_code
                                code_saved set_code_saved
                                on_sign=Box::new(move || {
                                    let dd = deploy_data.clone();
                                    let gd = gas_deploy.clone();
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
                                })
                            />
                        }.into_any()
                    },

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