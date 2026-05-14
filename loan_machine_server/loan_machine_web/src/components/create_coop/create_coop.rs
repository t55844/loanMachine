// loan_machine_web/src/components/create_coop.rs
//
// Six-step flow that chains two server calls and two on-chain txs:
//
//   Form           → user fills coop config, submits
//   AccessCode     → prepare action returned a deploy bundle; show the
//                    access code; user must save it, then click sign
//   WaitingDeploy  → TX 1 (deploy LoanMachine) is in flight on-chain
//   SignInit       → TX 1 confirmed; bridge gave us a contract address;
//                    user clicks to sign TX 2 (initializeMultisig)
//   Registering    → TX 2 confirmed; register action is in flight
//   Done           → cooperative is fully registered; show details
//
// Architecture notes (matches the patterns from the study notes):
//   - All JS interop is behind `privy_bridge`.  This file does not
//     import any wasm-bindgen types.
//   - Both server fns are `Action`s.  No `spawn_local`, no hand-rolled
//     loading / error / reg_started signals.
//   - Bridge events arrive via `on_tx_outcome` (TX 2) and
//     `on_deploy_outcome` (TX 1).  Cleanup is automatic on unmount.
//   - The two action settlements are handled in `Effect`s that read
//     `action.value()` and react to None / Some(Ok) / Some(Err).

use leptos::prelude::*;
use loan_machine_models::{
    requests::{CreateCoopRequest, RegisterDeployedCoopRequest},
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

    let (name,   set_name)   = signal(String::new());
    let (admin2, set_admin2) = signal(String::new());
    let (admin3, set_admin3) = signal(String::new());

    let (name_err,   set_name_err)   = signal(String::new());
    let (admin2_err, set_admin2_err) = signal(String::new());
    let (admin3_err, set_admin3_err) = signal(String::new());

    // Single top-of-page error display.  Written from multiple sources
    // (prepare action, bridge events, register action) — easier to
    // funnel them all here than to derive from N separate sources.
    let (error, set_error) = signal(String::new());

    // Results from the two on-chain steps.
    let (contract_address, set_contract_address) = signal(String::new());
    let (reg_result, set_reg_result) = signal(Option::<CoopRegistrationResult>::None);

    // AccessCode checkbox.
    let (code_saved, set_code_saved) = signal(false);

    // ── Server-fn Actions ───────────────────────────────────
    let prepare = Action::new(|req: &CreateCoopRequest| {
        let req = req.clone();
        async move { prepare_create_coop(req).await }
    });

    // The register action takes () because its request is assembled
    // at dispatch time from current signal values.
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
    //
    // Always sets the target step; only clears `error` when moving
    // *forward* (so a rollback after a failure keeps the error visible
    // to the user on the screen they land on).
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
                // Defensive guard: only advance if still on Form.
                // The Effect could in principle re-fire if value()
                // updates while we're past Form (it won't in normal
                // flow, but cheap to be safe).
                if step.get_untracked() == CoopStep::Form {
                    advance(CoopStep::AccessCode);
                }
            }
            Err(e) => set_error.set(e.to_string()),
        }
    });

    // ── TX 1 outcome (contract deployment) ──────────────────
    privy_bridge::on_deploy_outcome(move |outcome| match outcome {
        DeployOutcome::Complete { contract_address: addr, .. } => {
            set_contract_address.set(addr);
            advance(CoopStep::SignInit);
        }
        DeployOutcome::Failed(err) => {
            set_error.set(format!("Falha no deploy: {err}"));
            advance(CoopStep::AccessCode);
        }
    });

    // ── TX 2 outcome (multisig init) ────────────────────────
    //
    // No more step-guard like the old code had.  Bridge subscriptions
    // are mount-scoped, so vinculation's `on_tx_outcome` listener
    // can't trigger us — different routes can't be mounted at once.
    privy_bridge::on_tx_outcome(move |outcome| match outcome {
        TxOutcome::Complete(_hash) => {
            advance(CoopStep::Registering);
            register.dispatch(());
        }
        TxOutcome::Failed(err) => {
            set_error.set(format!("Falha na inicialização: {err}"));
            // Stay on SignInit — user can retry.
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
                    <SectionTitle>"CRIAR COOPERATIVA"</SectionTitle>
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
                            admin3 set_admin3
                            name_err admin2_err admin3_err
                            loading
                            founder_wallet=founder_wallet
                            on_submit=Box::new(move || {
                                let n  = name.get_untracked();
                                let a2 = admin2.get_untracked();
                                let a3 = admin3.get_untracked();

                                let a2_parsed: Result<WalletAddress, _> = a2.parse();
                                let a3_parsed: Result<WalletAddress, _> = a3.parse();

                                let name_ok   = !n.trim().is_empty();
                                let admin2_ok = a2_parsed.is_ok();
                                let admin3_ok = a3_parsed.is_ok();

                                set_name_err.set(if name_ok     { String::new() } else { "Nome obrigatório".into() });
                                set_admin2_err.set(if admin2_ok { String::new() } else { "Endereço inválido (0x + 40 hex)".into() });
                                set_admin3_err.set(if admin3_ok { String::new() } else { "Endereço inválido (0x + 40 hex)".into() });

                                if let (true, Ok(a2_addr), Ok(a3_addr)) = (name_ok, a2_parsed, a3_parsed) {
                                    // Dispatch the prepare Action — its
                                    // pending state drives the button's
                                    // loading prop automatically.
                                    prepare.dispatch(CreateCoopRequest {
                                        name:          n,
                                        founder_wallet,
                                        admin_wallets: vec![founder_wallet, a2_addr, a3_addr],
                                        threshold:     2,
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
                                        title: "CONFIRMAR DEPLOY".into(),
                                        estimates: vec![GasEstimate {
                                            label:   "Deploy do LoanMachine".into(),
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
                            label="Aguardando confirmação do deploy…"
                            hint="Assine a transação na sua carteira Privy"
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
                                        title: "CONFIRMAR INICIALIZAÇÃO".into(),
                                        estimates: vec![GasEstimate {
                                            label:   "Inicializar Multisig".into(),
                                            gas_hex: g.clone(),
                                        }],
                                        on_confirm: Callback::new(move |_| {
                                            // Pin the server's gas estimate
                                            // to skip an eth_estimateGas RPC
                                            // on the JS side.
                                            privy_bridge::send_tx(&a, &d, Some(&g));
                                        }),
                                    }));
                                })
                            />
                        }.into_any()
                    },

                    CoopStep::Registering => view! {
                        <WaitingStep
                            label="Registrando cooperativa na blockchain…"
                            hint="O servidor está assinando a transação de registro"
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