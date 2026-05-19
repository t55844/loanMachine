// loan_machine_web/src/components/elections/elections.rs
//
// Page wrapper.  Owns only:
//   • the Resource that reads current election state from the chain.
//   • a refresh trigger bumped by CurrentElection on tx success.
//
// Everything to do with the open-election form — signals, action, gas modal,
// bridge subscription — lives in CurrentElection, because that's the domain
// that owns it.  The only contract between page and child is an on_tx_success
// Callback<()> that tells the page "something changed on-chain, re-read."

use leptos::prelude::*;

use loan_machine_models::wallet_address::WalletAddress;

use crate::components::ui::*;
use crate::components::elections::current_election::CurrentElection;
use crate::components::elections::last_result::LastResult;
use crate::server_fns::elections::get_current_election;

#[component]
pub fn ElectionsPage(
    #[prop(into)] coop_id:       String,
    #[prop(into)] caller_wallet: WalletAddress,
) -> impl IntoView {
    let (refresh, set_refresh) = signal(0u32);

    let coop_id_resource = coop_id.clone();
    let current_election = Resource::new(
        move || (coop_id_resource.clone(), refresh.get()),
        |(id, _)| async move { get_current_election(id).await },
    );

    view! {
        <Section>
            <div class="container-md">
                <div class="t-center" style="margin-bottom: var(--sp-8)">
                    <SectionTitle>"ELEIÇÕES DE MODERADOR"</SectionTitle>
                </div>

                <Suspense fallback=move || view! {
                    <Card>
                        <div class="flex-col flex-center gap-6" style="padding: var(--sp-12) 0">
                            <span class="spinner" style="width: 40px; height: 40px; border-width: 3px" />
                            <p class="t-mono-xs t-muted">"Carregando estado da eleição…"</p>
                        </div>
                    </Card>
                }>
                    {move || current_election.get().map(|res| match res {
                        Ok(maybe) => view! {
                            <CurrentElection
                                coop_id=coop_id.clone()
                                caller_wallet
                                current=maybe
                                on_tx_success=Callback::new(move |_| {
                                    set_refresh.update(|n| *n += 1);
                                })
                            />
                        }.into_any(),
                        Err(e) => view! {
                            <Alert kind=AlertKind::Error>{e.to_string()}</Alert>
                        }.into_any(),
                    })}
                </Suspense>

                <div style="margin-top: var(--sp-6)">
                    <LastResult last=None />
                </div>
            </div>
        </Section>
    }
}