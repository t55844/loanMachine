// src/app.rs
use leptos::*;
use leptos::task::spawn_local;
use leptos::prelude::ElementChild;
use crate::models::responses::TransactionResponse;
use leptos::prelude::*; 
use crate::components::ui::*;



#[server(PrepareVinculation, "/api")]
pub async fn prepare_vinculation(
    member_id: u32, 
    wallet_address: String
) -> Result<TransactionResponse, ServerFnError> {  // ServerFnError with no generic = uses default
    
    let state = use_context::<crate::config::AppState>()
        .ok_or_else(|| ServerFnError::new("AppState not found in context"))?;
    
    let bc = &state.blockchain_service;
    
    let wallet: alloy::primitives::Address = wallet_address.parse()
        .map_err(|_| ServerFnError::new("Invalid wallet address"))?;

    let data = bc.encode_vinculation_member(member_id, wallet);
    
    let gas_estimate: alloy::primitives::U256 = bc
        .estimate_vinculation_member_to_wallet_gas(member_id, wallet)
        .await
        .map_err(|e| ServerFnError::new(format!("Gas estimation error: {}", e)))?;

    Ok(TransactionResponse {
        to: state.contract_address.0.clone(),
        data: format!("0x{}", hex::encode(data.as_ref())),
        value: "0".to_string(),
        gas_estimate: gas_estimate.to_string(),
    })
}

 #[component]
 pub fn App() -> impl IntoView {
     view! {
         <link rel="stylesheet" href="/style/design-system.css"/>
         <PageLayout title="LOAN MACHINE">
             <Ticker items=vec![
                 ("ETH",  "$3,241.00", "2.4%",  true),
                 ("BTC",  "$67,100",   "1.1%",  true),
                 ("GAS",  "12 gwei",   "0.5%",  false),
                 ("USDT", "$1.00",     "0.01%", true),
             ]/>
             <Section>
                 <SectionTitle>"Vinculate Member"</SectionTitle>
                 <VinculationForm />
             </Section>
         </PageLayout>
     }
 }

 #[component]
 fn VinculationForm() -> impl IntoView {
     let (member_id, set_member_id) = signal(0u32);
     let (wallet, set_wallet) = signal("".to_string());
     let (loading, set_loading) = signal(false);
     let (result, set_result) = signal::<Option<TransactionResponse>>(None);
     let (error, set_error) = signal::<Option<String>>(None);

     let on_submit = move |ev: leptos::ev::SubmitEvent| {
         ev.prevent_default();
         let id = member_id.get();
         let addr = wallet.get();
         set_loading.set(true);
         set_error.set(None);
         spawn_local(async move {
             match prepare_vinculation(id, addr).await {
                 Ok(tx) => { set_result.set(Some(tx)); }
                 Err(e) => { set_error.set(Some(e.to_string())); }
             }
             set_loading.set(false);
         });
     };

     view! {
         <div class="grid-2" style="gap: var(--sp-8); align-items: start">
             // LEFT — form card
             <Card variant=CardVariant::Yellow tag="STEP 01" hover=true>
                 <form on:submit=on_submit style="display: flex; flex-direction: column; gap: var(--sp-6); margin-top: var(--sp-4)">
                     <NumberInput
                         label="Member ID"
                         value=member_id
                         set_value=set_member_id
                         hint="Your cooperative member number"
                     />
                     <AddressInput
                         label="Wallet Address"
                         value=wallet
                         set_value=set_wallet
                         error=error.get().unwrap_or_default()
                     />
                     <Button variant=BtnVariant::Primary size=BtnSize::Lg full_width=true loading=loading.get()>
                         "VINCULATE WALLET"
                     </Button>
                 </form>
             </Card>
             // RIGHT — result card
             <div>
                 <Card variant=CardVariant::Default hover=false>
                     <h3 class="t-display-md t-yellow">"HOW IT WORKS"</h3>
                     <div class="mt-6" style="display:flex;flex-direction:column;gap:var(--sp-4)">
                         <div class="step">
                             <span class="step-number">"01"</span>
                             <div>
                                 <p class="form-label">"Enter your ID"</p>
                                 <p class="t-mono-sm t-muted">"Your cooperative member number issued at registration."</p>
                             </div>
                         </div>
                         <div class="step">
                             <span class="step-number">"02"</span>
                             <div>
                                 <p class="form-label">"Paste wallet address"</p>
                                 <p class="t-mono-sm t-muted">"Any EVM wallet: MetaMask, WalletConnect, etc."</p>
                             </div>
                         </div>
                         <div class="step">
                             <span class="step-number">"03"</span>
                             <div>
                                 <p class="form-label">"Sign the transaction"</p>
                                 <p class="t-mono-sm t-muted">"Your wallet will prompt for gas fee approval."</p>
                             </div>
                         </div>
                     </div>
                 </Card>
                 {move || result.get().map(|tx| view! {
                     <TxCard
                         to=tx.to
                         data=tx.data
                         gas_estimate=tx.gas_estimate
                     />
                 })}
                 {move || error.get().map(|e| view! {
                     <div class="mt-4">
                         <Alert kind=AlertKind::Error>
                             {e}
                         </Alert>
                     </div>
                 })}
             </div>
         </div>
     }
 }