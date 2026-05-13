// src/components/vinculation.rs
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::ev::SubmitEvent;
use leptos::web_sys; 
use wasm_bindgen::JsCast;

use crate::components::ui::*;
use loan_machine_models::responses::{CoopInfo, VinculationBundle,};
use loan_machine_models::requests::{DocKind};
use loan_machine_models::wallet_address::WalletAddress;

use crate::server_fns::vinculation::{get_wallet_coop, prepare_first_vinculation};
// ── GATE ─────────────────────────────────────────────────────
#[derive(Clone, PartialEq)]
pub enum TxStatus {
    Idle,                    // nothing happening
    Pending,                 // waiting for Privy to sign + submit
    Complete(String),        // String = the tx hash "0x123..."
    Failed(String),          // String = error message
}
#[component]
pub fn VinculationGate(
    smart_wallet: WalletAddress,
    children: ChildrenFn,
) -> impl IntoView {
    let (status, set_status) = signal(GateStatus::Checking);
    let wallet = smart_wallet.clone();

    spawn_local(async move {
        match get_wallet_coop(wallet).await {
            Ok(Some(coop)) => set_status.set(GateStatus::Vinculated(coop)),
            Ok(None)       => set_status.set(GateStatus::NeedsVinculation),
            Err(e)         => set_status.set(GateStatus::Error(e.to_string())),
        }
    });

    view! {
        {move || match status.get() {

            GateStatus::Checking => view! {
                <div class="flex-center" style="height: 60vh">
                    <div class="flex-col flex-center gap-6">
                        <div class="spinner"></div>
                        <p class="t-mono-sm t-muted">"Verificando vínculo..."</p>
                    </div>
                </div>
            }.into_any(),

            GateStatus::NeedsVinculation => view! {
                <section class="section">
                    <div class="container-sm">
                        <FirstVinculationForm
                            smart_wallet=smart_wallet.clone()
                            on_success=move |coop| set_status.set(GateStatus::Vinculated(coop))
                        />
                    </div>
                </section>
            }.into_any(),

            GateStatus::Vinculated(_) => children().into_any(),

            GateStatus::Error(e) => view! {
                <section class="section">
                    <div class="container-sm">
                        <Alert kind=AlertKind::Error>{e}</Alert>
                    </div>
                </section>
            }.into_any(),
        }}
    }
}

#[derive(Clone)]
enum GateStatus {
    Checking,
    NeedsVinculation,
    Vinculated(CoopInfo),
    Error(String),
}
// ── FIRST VINCULATION FORM ────────────────────────────────────

#[component]
pub fn FirstVinculationForm(
    smart_wallet: WalletAddress,
    on_success:   impl Fn(CoopInfo) + 'static,
) -> impl IntoView {
    let (loading,     set_loading)     = signal(false);
    let (error,       set_error)       = signal::<Option<String>>(None);
    let (bundle,      set_bundle)      = signal::<Option<VinculationBundle>>(None);
    let (tx_status, set_tx_status) = signal(TxStatus::Idle);

    let wallet = smart_wallet.clone();


    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::{JsCast, JsValue, closure::Closure};

        let on_complete = Closure::<dyn Fn(web_sys::CustomEvent)>::new(move |e: web_sys::CustomEvent| {
            let hash = js_sys::Reflect::get(&e.detail(), &JsValue::from_str("tx_hash"))
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_default();
            set_tx_status.set(TxStatus::Complete(hash));
        });
        web_sys::window().unwrap()
            .add_event_listener_with_callback(
                "privy_tx_complete",
                on_complete.as_ref().unchecked_ref(),
            ).ok();
        on_complete.forget();

        let on_error = Closure::<dyn Fn(web_sys::CustomEvent)>::new(move |e: web_sys::CustomEvent| {
            let err = js_sys::Reflect::get(&e.detail(), &JsValue::from_str("error"))
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_else(|| "unknown".into());
            set_tx_status.set(TxStatus::Failed(err));
        });
        web_sys::window().unwrap()
            .add_event_listener_with_callback(
                "privy_tx_error",
                on_error.as_ref().unchecked_ref(),
            ).ok();
        on_error.forget();
    }
   
        view! {
            <div class="flex-col gap-8">

            <IdentifyCard
                error=error
                loading=loading
                on_submit=Callback::new(move |(kind, document, cid, code):
                    (DocKind, String, String, String)| {
                    set_loading.set(true);
                    set_error.set(None);
                    let w = wallet.clone();
                    spawn_local(async move {
                        match prepare_first_vinculation(kind, document, w, cid, code).await {
                            Ok(b)  => set_bundle.set(Some(b)),
                            Err(e) => set_error.set(Some(e.to_string())),
                        }
                        set_loading.set(false);
                    });
                })
            />

        // ── CARD 2: sign button — only shown when bundle exists ──
        // This card disappears once tx is sent (tx_status != Idle)
            {move || bundle.get().map(|b| {
                if tx_status.get() != TxStatus::Idle {
                    return view! { <div></div> }.into_any();
                }
                view! {
                    <SignCard bundle=b set_tx_status=set_tx_status />
                }.into_any()
            })}

        // ── CARD 3: tx status — only shown after sign clicked ──
        // This reads tx_status signal and re-renders on every change
        <TxStatusCard tx_status=tx_status set_tx_status=set_tx_status />


    
    </div>
    }

}

#[component]
fn IdentifyCard(
    on_submit: Callback<(DocKind, String, String, String)>,
    error: ReadSignal<Option<String>>,
    loading: ReadSignal<bool>,

) -> impl IntoView {

    let (doc_kind, set_doc_kind) = signal(DocKind::Cpf);
    let (document, set_document) = signal(String::new());
    let (coop_id,     set_coop_id)     = signal(String::new());
    let (access_code, set_access_code) = signal(String::new());

    let switch_to = move |kind: DocKind|{
        set_doc_kind.set(kind);
        set_document.set(String::new());
    };

    let on_document_input = move |ev: leptos::ev::Event|{
        let target = ev.target()
        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());

        let Some(input) = target else { return };

        let raw = input.value();
        let max_len = doc_kind.get().max_input_len();

        let cleaned: String = raw
        .chars()
        .filter(|c| c.is_ascii_digit() || matches!(c,'.'|'-'|'/' ))
        .take(max_len)
        .collect();

        if cleaned != raw{
            input.set_value(&cleaned);
        }
        set_document.set(cleaned);
    };

    let on_prepare = move |ev: SubmitEvent|{
        ev.prevent_default();
        on_submit.run((
            doc_kind.get(),
            document.get(),
            coop_id.get(),
            access_code.get()
        ));
    };

      view! {
        // ── TITLE ──────────────────────────────────────────
        <div class="t-center">
            <h1 class="t-display-lg t-yellow">"VINCULE SUA CARTEIRA"</h1>
            <p class="t-mono-sm t-muted mt-4">
                "Conecte seu documento ao seu endereço blockchain."
            </p>
        </div>

        <Card variant=CardVariant::Yellow tag="PASSO 01 — IDENTIFICAÇÃO" hover=false>
            <form
                on:submit=on_prepare
                style="display:flex; flex-direction:column; gap:var(--sp-6); margin-top:var(--sp-4)"
            >
                // ── CPF / CNPJ toggle ──────────────────────
                <div class="doc-kind-toggle" role="tablist">
                    <button
                        type="button"
                        role="tab"
                        class=move || {
                            if doc_kind.get() == DocKind::Cpf {
                                "toggle-btn toggle-btn-active"
                            } else {
                                "toggle-btn"
                            }
                        }
                        on:click=move |_| switch_to(DocKind::Cpf)
                    >
                        "CPF"
                    </button>
                    <button
                        type="button"
                        role="tab"
                        class=move || {
                            if doc_kind.get() == DocKind::Cnpj {
                                "toggle-btn toggle-btn-active"
                            } else {
                                "toggle-btn"
                            }
                        }
                        on:click=move |_| switch_to(DocKind::Cnpj)
                    >
                        "CNPJ"
                    </button>
                </div>

                // ── Document input ─────────────────────────
                <div class="form-group">
                    <label class="form-label">
                        {move || doc_kind.get().label()}
                    </label>
                    <input
                        class="form-input"                       
                        type="text"
                        inputmode="numeric"
                        autocomplete="off"
                        prop:value=document
                        placeholder=move || doc_kind.get().placeholder()
                        maxlength=move || doc_kind.get().max_input_len() as i32
                        on:input=on_document_input
                    />
                    <span class="form-hint">                       
                        {move || match doc_kind.get() {
                            DocKind::Cpf  => "11 dígitos — formatação opcional",
                            DocKind::Cnpj => "14 dígitos — formatação opcional",
                        }}
                    </span>
                </div>

                <TextInput
                    label="ID da Cooperativa"
                    placeholder="0xabc123..."
                    hint="O identificador bytes32 da sua cooperativa"
                    value=coop_id
                    set_value=set_coop_id
                    // error prop omitted → defaults to empty Signal
                />

                <TextInput
                    label="Código de Acesso"
                    placeholder="Fornecido pelo administrador"
                    value=access_code
                    set_value=set_access_code
                    error=Signal::derive(move || error.get().unwrap_or_default())
                />


                <Button
                    variant=BtnVariant::Primary
                    size=BtnSize::Lg
                    full_width=true
                    loading=loading        // ← ReadSignal<bool> passes directly via `into`
                >
                    "PREPARAR VINCULAÇÃO"
                </Button>
            </form>
        </Card>
    }
}


#[component]
fn SignCard(
    bundle:        VinculationBundle,
    set_tx_status: WriteSignal<TxStatus>,
) -> impl IntoView {
    let b = bundle.clone();

    view! {
        <Card variant=CardVariant::Gold tag="PASSO 02 — ASSINAR" hover=false>
            <div class="flex-col gap-6 mt-4">
                <p class="t-mono-sm t-muted">
                    "Uma transação vinculará seu ID de membro à sua carteira."
                </p>
                <div class="stat-block">
                    <span class="stat-label">"Loan Machine"</span>
                    <HashDisplay value=bundle.loan_machine_address.clone() />
                </div>
                <div class="stat-block">
                    <span class="stat-label">"Gas Estimado"</span>
                    <span class="stat-value-sm">{bundle.gas_join.clone()}" gas"</span>
                </div>
                <Button
                    variant=BtnVariant::Primary
                    size=BtnSize::Lg
                    full_width=true
                    on_click=Box::new(move || {
                        set_tx_status.set(TxStatus::Pending);
                        send_bundle_to_privy(b.clone());
                    })
                >
                    "ASSINAR COM SUA CARTEIRA"
                </Button>
            </div>
        </Card>
    }
}


#[component]
fn TxStatusCard(
    tx_status: ReadSignal<TxStatus>,
    set_tx_status: WriteSignal<TxStatus>,
) -> impl IntoView {
    move || match tx_status.get() {

        TxStatus::Idle => view! { <div></div> }.into_any(),

        TxStatus::Pending => view! {
            <Card variant=CardVariant::Default hover=false>
                <div class="flex-center gap-4" style="padding: var(--sp-8) 0">
                    <span class="spinner"></span>
                    <span class="t-mono-sm t-muted">
                        "Aguardando assinatura e confirmação..."
                    </span>
                </div>
            </Card>
        }.into_any(),

        TxStatus::Complete(hash) => view! {
            <Card variant=CardVariant::Gold tag="VINCULAÇÃO ENVIADA" hover=false>
                <div class="flex-col gap-4 mt-4">
                    <Alert kind=AlertKind::Success>
                        "Transação enviada com sucesso!"
                    </Alert>
                    <div class="stat-block">
                        <span class="stat-label">"Transaction Hash"</span>
                        <HashDisplay value=hash />
                    </div>
                    <p class="t-mono-xs t-muted">
                        "Aguarde a confirmação na blockchain. \
                         Isso pode levar 10–30 segundos."
                    </p>
                </div>
            </Card>
        }.into_any(),

        TxStatus::Failed(err) => view! {
            <Card variant=CardVariant::Default hover=false>
                <div class="flex-col gap-4 mt-4">
                    <Alert kind=AlertKind::Error>
                        "Falha ao enviar transação."
                    </Alert>
                    <p class="t-mono-sm t-muted">{err}</p>
                    <Button
                        variant=BtnVariant::Ghost
                        on_click=Box::new(move || {
                            set_tx_status.set(TxStatus::Idle);
                        })
                    >
                        "TENTAR NOVAMENTE"
                    </Button>
                </div>
            </Card>
        }.into_any(),
    }
}

// ── JS INTEROP ────────────────────────────────────────────────

fn send_bundle_to_privy(bundle: VinculationBundle) {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsValue;
        use js_sys::Function;
        use wasm_bindgen::JsCast;

        // Map domain bundle → bridge's {to, data} shape
        let json = format!(
            r#"{{"to":"{}","data":"{}"}}"#,
            bundle.loan_machine_address,
            bundle.join_calldata,
        );

        let window = web_sys::window().unwrap();
        if let Ok(val) = js_sys::Reflect::get(&window, &JsValue::from_str("loan_machine_send_tx")) {
            if let Ok(func) = val.dyn_into::<Function>() {
                let arg = JsValue::from_str(&json);
                let _ = func.call1(&JsValue::NULL, &arg);
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = bundle;
}
