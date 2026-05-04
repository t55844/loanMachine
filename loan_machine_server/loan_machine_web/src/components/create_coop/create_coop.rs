// loan_machine_web/src/components/create_coop.rs
//
// All logic for the create-coop flow:
//   - CoopStep state machine
//   - JS interop (WASM-only helpers)
//   - CreateCoopPage: signals, event listeners, registration effect, step dispatch

use leptos::prelude::*;
use leptos::task::spawn_local;
use loan_machine_models::{
    requests::{CreateCoopRequest, RegisterDeployedCoopRequest},
    responses::{CoopDeployBundle, CoopRegistrationResult},
};

use crate::components::ui::*;
use crate::components::create_coop::create_coop_steps::{
    AccessCodeStep, DoneStep, FormStep, SignInitStep, StepProgress, WaitingStep,
};
use crate::server_fns::create_coop::{prepare_create_coop, register_deployed_coop};

// ── Step state machine ────────────────────────────────────────
// pub so create_coop_steps can import it for StepProgress.

#[derive(Clone, PartialEq)]
pub enum CoopStep {
    Form,
    AccessCode,
    WaitingDeploy,
    SignInit,
    Registering,
    Done,
}

// ── JS interop ───────────────────────────────────────────────
//
// Both functions cross the Rust→JS boundary via wasm-bindgen's Reflect API:
//
//   js_sys::Reflect::get(&window, "name")  →  window["name"]         (runtime lookup)
//   val.dyn_into::<Function>()             →  assert typeof === "function"
//   func.call2(&JsValue::NULL, a, b)       →  func.call(null, a, b)
//
// JsValue::NULL is the `this` context — equivalent to calling as a standalone fn.
// The cfg gate compiles these bodies away entirely on the SSR server (Linux x86_64).

/// TX 1 — deploy LoanMachine (no `to` = EVM deployment).
/// Calls window.loan_machine_deploy_contract(data, gas).
/// Bridge fires privy_deploy_complete { contract_address } on receipt.
fn js_deploy_contract(data: String, gas: String) {
    #[cfg(target_arch = "wasm32")]
    {
        use js_sys::Function;
        use wasm_bindgen::{JsCast, JsValue};
        let window = web_sys::window().unwrap();
        if let Ok(v) = js_sys::Reflect::get(&window, &JsValue::from_str("loan_machine_deploy_contract")) {
            if let Ok(f) = v.dyn_into::<Function>() {
                let _ = f.call2(&JsValue::NULL, &JsValue::from_str(&data), &JsValue::from_str(&gas));
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (data, gas);
}

/// TX 2 — call initializeMultisig on the deployed contract.
/// Calls window.loan_machine_send_tx(json) where json = { to, data, gas }.
/// Bridge fires privy_tx_complete { tx_hash } on receipt.
fn js_send_tx(to: String, data: String, gas: String) {
    let json = format!(r#"{{"to":"{to}","data":"{data}","gas":"{gas}"}}"#);
    #[cfg(target_arch = "wasm32")]
    {
        use js_sys::Function;
        use wasm_bindgen::{JsCast, JsValue};
        let window = web_sys::window().unwrap();
        if let Ok(v) = js_sys::Reflect::get(&window, &JsValue::from_str("loan_machine_send_tx")) {
            if let Ok(f) = v.dyn_into::<Function>() {
                let _ = f.call1(&JsValue::NULL, &JsValue::from_str(&json));
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = json;
}

// ── CreateCoopPage ────────────────────────────────────────────

#[component]
pub fn CreateCoopPage(#[prop(into)] founder_wallet: String) -> impl IntoView {

    // ── Signals ──────────────────────────────────────────────

    let (step, set_step) = signal(CoopStep::Form);

    let (name,   set_name)   = signal(String::new());
    let (admin2, set_admin2) = signal(String::new());
    let (admin3, set_admin3) = signal(String::new());

    let (name_err,   set_name_err)   = signal(String::new());
    let (admin2_err, set_admin2_err) = signal(String::new());
    let (admin3_err, set_admin3_err) = signal(String::new());

    let (bundle,           set_bundle)           = signal(Option::<CoopDeployBundle>::None);
    let (contract_address, set_contract_address) = signal(String::new());
    let (reg_result,       set_reg_result)       = signal(Option::<CoopRegistrationResult>::None);

    let (code_saved,  set_code_saved)  = signal(false);
    let (loading,     set_loading)     = signal(false);
    let (error,       set_error)       = signal(String::new());
    let (reg_started, set_reg_started) = signal(false);

    // StoredValue: plain String prop, not reactive — no clone on every render.
    let founder = StoredValue::new(founder_wallet);

    // ── JS event listeners ────────────────────────────────────
    // Registered in component body (not in Effect) so they fire exactly once.
    // Same pattern as WalletRouter session-restore in app.rs.

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::{JsCast, JsValue, closure::Closure};

        // privy_deploy_complete → TX 1 mined, bridge returned contract address
        let cl = Closure::<dyn Fn(web_sys::CustomEvent)>::new(move |e: web_sys::CustomEvent| {
            let addr = js_sys::Reflect::get(&e.detail(), &JsValue::from_str("contract_address"))
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_default();
            if !addr.is_empty() {
                set_contract_address.set(addr);
                set_step.set(CoopStep::SignInit);
            }
        });
        web_sys::window().unwrap()
            .add_event_listener_with_callback("privy_deploy_complete", cl.as_ref().unchecked_ref())
            .ok();
        cl.forget();

        // privy_tx_complete → TX 2 signed; guard on step so vinculation's
        // listener (which fires the same event) doesn't accidentally advance us.
        let cl2 = Closure::<dyn Fn(web_sys::CustomEvent)>::new(move |_e: web_sys::CustomEvent| {
            if step.get_untracked() == CoopStep::SignInit {
                set_step.set(CoopStep::Registering);
            }
        });
        web_sys::window().unwrap()
            .add_event_listener_with_callback("privy_tx_complete", cl2.as_ref().unchecked_ref())
            .ok();
        cl2.forget();

        // privy_tx_error → bridge surfaced an error from either tx
        let cl3 = Closure::<dyn Fn(web_sys::CustomEvent)>::new(move |e: web_sys::CustomEvent| {
            let msg = js_sys::Reflect::get(&e.detail(), &JsValue::from_str("error"))
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_else(|| "Erro desconhecido".into());

            set_error.set(format!("Falha na transação: {msg}"));

            // Roll back the step so the user can retry
            match step.get_untracked() {
                CoopStep::WaitingDeploy => set_step.set(CoopStep::AccessCode),
                CoopStep::SignInit      => {},                 // already on the retry screen
                _                       => {},
            }
        });
        web_sys::window().unwrap()
            .add_event_listener_with_callback("privy_tx_error", cl3.as_ref().unchecked_ref())
            .ok();
        cl3.forget();
    }

    // ── Registration effect ───────────────────────────────────
    // reg_started guard prevents double-dispatch if Leptos re-runs the effect.

    Effect::new(move |_| {
        if step.get() == CoopStep::Registering && !reg_started.get_untracked() {
            set_reg_started.set(true);
            let req = RegisterDeployedCoopRequest {
                name:                 name.get_untracked(),
                loan_machine_address: contract_address.get_untracked(),
                founder_wallet:       founder.get_value(),
            };
            spawn_local(async move {
                match register_deployed_coop(req).await {
                    Ok(r)  => { set_reg_result.set(Some(r)); set_step.set(CoopStep::Done); }
                    Err(e) => {
                        set_error.set(e.to_string());
                        set_step.set(CoopStep::Form);
                        set_reg_started.set(false);
                    }
                }
            });
        }
    });

    // ── Step dispatch ─────────────────────────────────────────

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
                            founder_wallet=founder.get_value()
                            on_submit=Box::new(move || {
                                let n  = name.get_untracked();
                                let a2 = admin2.get_untracked();
                                let a3 = admin3.get_untracked();
                                let valid_addr = |a: &str| a.starts_with("0x") && a.len() == 42;
                                let mut ok = true;

                                if n.trim().is_empty() {
                                    set_name_err.set("Nome obrigatório".into()); ok = false;
                                } else { set_name_err.set(String::new()); }

                                if !valid_addr(&a2) {
                                    set_admin2_err.set("Endereço inválido (0x + 40 hex)".into()); ok = false;
                                } else { set_admin2_err.set(String::new()); }

                                if !valid_addr(&a3) {
                                    set_admin3_err.set("Endereço inválido (0x + 40 hex)".into()); ok = false;
                                } else { set_admin3_err.set(String::new()); }

                                if ok {
                                    set_loading.set(true);
                                    set_error.set(String::new());
                                    let req = CreateCoopRequest {
                                        name:          n,
                                        founder_wallet: founder.get_value(),
                                        admin_wallets: vec![founder.get_value(), a2, a3],
                                        threshold:     2,
                                    };
                                    spawn_local(async move {
                                        match prepare_create_coop(req).await {
                                            Ok(b)  => { set_bundle.set(Some(b)); set_step.set(CoopStep::AccessCode); }
                                            Err(e) => { set_error.set(e.to_string()); }
                                        }
                                        set_loading.set(false);
                                    });
                                }
                            })
                        />
                    }.into_any(),

                    CoopStep::AccessCode => {
                        let access_code = bundle.get().map(|b| b.access_code.clone()).unwrap_or_default();
                        let deploy_data = bundle.get().map(|b| b.deploy_data.clone()).unwrap_or_default();
                        let gas_deploy  = bundle.get().map(|b| b.gas_deploy.clone()).unwrap_or_default();
                        view! {
                            <AccessCodeStep
                                access_code
                                code_saved set_code_saved
                                on_sign=Box::new(move || {
                                    set_step.set(CoopStep::WaitingDeploy);
                                    js_deploy_contract(deploy_data.clone(), gas_deploy.clone());
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
                        view! {
                            <SignInitStep
                                contract_address=addr
                                on_sign=Box::new(move || {
                                    js_send_tx(addr_tx.clone(), init_data.clone(), gas_init.clone());
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