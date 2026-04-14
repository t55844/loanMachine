// src/components/vinculation.rs
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::ev::SubmitEvent;

use crate::components::ui::*;
use loan_machine_models::responses::{CoopInfo, VinculationBundle};

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
    smart_wallet: String,
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
    smart_wallet: String,
    on_success:   impl Fn(CoopInfo) + 'static,
) -> impl IntoView {
    let (loading,     set_loading)     = signal(false);
    let (error,       set_error)       = signal::<Option<String>>(None);
    let (bundle,      set_bundle)      = signal::<Option<VinculationBundle>>(None);
    let (tx_status, set_tx_status) = signal(TxStatus::Idle);

    let wallet = smart_wallet.clone();

   
        view! {
            <div class="flex-col gap-8">

            <IdentifyCard  error=error loading=loading 
                on_submit=move |id, cid, code| {
                set_loading.set(true);
                set_error.set(None);
                let w = wallet.clone();
                spawn_local(async move {
                    match prepare_first_vinculation(id, w, cid, code).await {
                        Ok(b)  => set_bundle.set(Some(b)),
                        Err(e) => set_error.set(Some(e.to_string())),
                    }
                    set_loading.set(false);
                });
            }/>

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
    on_submit:   impl Fn(u32, String, String) + Send + Sync + 'static,
    error: ReadSignal<Option<String>>,
    loading: ReadSignal<bool>,

) -> impl IntoView {

    let (member_id,   set_member_id)   = signal(0u32);
    let (coop_id,     set_coop_id)     = signal(String::new());
    let (access_code, set_access_code) = signal(String::new());

    let on_prepare = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        on_submit(member_id.get(), coop_id.get(), access_code.get());
    };

     view! {

        // ── TITLE ──────────────────────────────────────────
        <div class="t-center">
            <h1 class="t-display-lg t-yellow">"VINCULE SUA CARTEIRA"</h1>
            <p class="t-mono-sm t-muted mt-4">
                "Conecte seu ID de membro ao seu endereço blockchain."
            </p>
        </div>

        // ── CARD 1: form inputs (unchanged) ────────────────
        <Card variant=CardVariant::Yellow tag="PASSO 01 — IDENTIFICAÇÃO" hover=false>
            <form
                on:submit=on_prepare
                style="display:flex; flex-direction:column; gap:var(--sp-6); margin-top:var(--sp-4)"
            >
                <NumberInput
                    label="ID do Membro"
                    value=member_id
                    set_value=set_member_id
                    hint="Seu número de membro na cooperativa"
                />
                <TextInput
                    label="ID da Cooperativa"
                    placeholder="0xabc123..."
                    hint="O identificador bytes32 da sua cooperativa"
                    value=coop_id
                    set_value=set_coop_id
                    error=String::new()
                />
                <TextInput
                    label="Código de Acesso"
                    placeholder="Fornecido pelo administrador"
                    hint=""
                    value=access_code
                    set_value=set_access_code
                    error=error.get().unwrap_or_default()
                />

                {move || error.get().map(|e| view! {
                    <Alert kind=AlertKind::Error>{e}</Alert>
                })}

                <Button
                    variant=BtnVariant::Primary
                    size=BtnSize::Lg
                    full_width=true
                    loading=loading.get()
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

        if let Ok(json) = serde_json::to_string(&bundle) {
            let window = web_sys::window().unwrap();
            if let Ok(val) = js_sys::Reflect::get(&window, &JsValue::from_str("loan_machine_send_tx")) {
                if let Ok(func) = val.dyn_into::<Function>() {
                    let arg = JsValue::from_str(&json);
                    let _ = func.call1(&JsValue::NULL, &arg);
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    let _ = bundle;
}


#[server(GetWalletCoop, "/api")]
pub async fn get_wallet_coop(
    smart_wallet: String,
) -> Result<Option<CoopInfo>, ServerFnError> {
    #[cfg(feature = "ssr")]
    use loan_machine_core::services::blockchain::BlockchainError;
    #[cfg(feature = "ssr")]
    use alloy::primitives::{Address, FixedBytes};

    let state = use_context::<loan_machine_core::config::AppState>()
        .ok_or_else(|| ServerFnError::new("AppState not found"))?;

    let wallet: Address = smart_wallet
        .parse()
        .map_err(|_| ServerFnError::new("Invalid wallet address"))?;

    // Explicit type: FixedBytes<32>
    let coop_id: FixedBytes<32> = state
        .blockchain_service
        .factory
        .get_wallet_coop(wallet)
        .await
        .map_err(|e: BlockchainError| ServerFnError::new(e.to_string()))?;

    if coop_id == FixedBytes::<32>::ZERO {
        return Ok(None);
    }

    // Explicit type: CoopInfo
    let info: CoopInfo = state
        .blockchain_service
        .factory
        .get_coop_info(coop_id)
        .await
        .map_err(|e: BlockchainError| ServerFnError::new(e.to_string()))?;

    Ok(Some(info))
}

#[server(PrepareFirstVinculation, "/api")]
pub async fn prepare_first_vinculation(
    member_id:    u32,
    smart_wallet: String,
    coop_id:      String,
    access_code:  String,
) -> Result<VinculationBundle, ServerFnError> {
    #[cfg(feature = "ssr")]
    use loan_machine_core::services::blockchain::BlockchainError;
    #[cfg(feature = "ssr")]
    use alloy::primitives::{Address, FixedBytes, U256};

    let state = use_context::<loan_machine_core::config::AppState>()
        .ok_or_else(|| ServerFnError::new("AppState not found"))?;

    let wallet: Address = smart_wallet
        .parse()
        .map_err(|_| ServerFnError::new("Invalid wallet address"))?;

    let coop_id_bytes: FixedBytes<32> = coop_id
        .parse()
        .map_err(|_| ServerFnError::new("Invalid coop ID format"))?;

    let factory = &state.blockchain_service.factory;

    // Explicit type: Address
    let loan_machine_addr: Address = factory
        .get_loan_machine(coop_id_bytes)
        .await
        .map_err(|e: BlockchainError| ServerFnError::new(e.to_string()))?;

    // Explicit type: bool
    let approved: bool = factory
        .is_wallet_approved(loan_machine_addr, wallet)
        .await
        .map_err(|e: BlockchainError| ServerFnError::new(e.to_string()))?;

    if !approved {
        return Err(ServerFnError::new(
            "Carteira não aprovada. Entre em contato com o administrador.",
        ));
    }

    verify_member_not_vinculated(&state.subgraph_url.0, member_id)
        .await
        .map_err(ServerFnError::new)?;

    let join_calldata = factory.encode_join_coop(member_id, wallet, &access_code);

    // Explicit type: U256
    let gas: U256 = factory
        .estimate_join_coop_gas(loan_machine_addr, member_id, wallet, &access_code)
        .await
        .map_err(|e: BlockchainError| ServerFnError::new(e.to_string()))?;

    Ok(VinculationBundle {
        join_calldata:        format!("0x{}", hex::encode(join_calldata.as_ref())),
        loan_machine_address: loan_machine_addr.to_string(),
        factory_address:      state.factory_address.0.clone(),
        gas_join:             gas.to_string(),
    })
}

// ── SUBGRAPH ──────────────────────────────────────────────────

#[cfg(feature = "ssr")]
async fn verify_member_not_vinculated(
    subgraph_url: &str,
    member_id:    u32,
) -> Result<(), String> {
    if member_id == 0 {
        return Err("Member ID cannot be zero".into());
    }

    let query = format!(r#"{{
        memberToWalletVinculations(where: {{ memberId: "{member_id}" }}) {{
            memberId
            wallet
        }}
    }}"#);

    let resp: serde_json::Value = reqwest::Client::new()
        .post(subgraph_url)
        .json(&serde_json::json!({ "query": query }))
        .send()
        .await
        .map_err(|e| format!("Subgraph unreachable: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Subgraph parse error: {e}"))?;

    let entries = resp["data"]["memberToWalletVinculations"]
        .as_array()
        .ok_or("Unexpected subgraph response")?;

    if !entries.is_empty() {
        return Err("Este ID de membro já está vinculado a uma carteira.".into());
    }

    Ok(())
}