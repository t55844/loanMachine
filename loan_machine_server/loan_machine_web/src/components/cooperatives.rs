// loan_machine_web/src/components/cooperatives.rs

use leptos::prelude::*;
use crate::components::ui::*;
use crate::server_fns::cooperatives::list_cooperatives;
use loan_machine_models::responses::CooperativeView;

#[component]
pub fn CooperativesPage(
    wallet: ReadSignal<Option<String>>,
) -> impl IntoView {
    let coops = LocalResource::new(move || {
        let connected = wallet.get().is_some();
        async move {
            if !connected {
                return Ok(Vec::<CooperativeView>::new());
            }

            let token = get_access_token_js().await.unwrap_or_default();
            if token.is_empty() {
                return Err(ServerFnError::new("no_token_available"));
            }

            list_cooperatives(token).await
        }
    });

    view! {
        <section class="section">
            <div class="container">
                <SectionTitle>"REGISTERED COOPERATIVES"</SectionTitle>
                <Suspense fallback=LoadingState>
                    {move || coops.get().map(|res| match res {
                        Ok(list) if list.is_empty() => view! {
                            <Alert kind=AlertKind::Info>
                                "No cooperatives registered yet."
                            </Alert>
                        }.into_any(),
                        Ok(list) => view! {
                            <div class="grid-2 mt-8">
                                {list.into_iter()
                                    .map(|c| view! { <CoopCard coop=c /> })
                                    .collect_view()}
                            </div>
                        }.into_any(),
                        Err(e) => view! {
                            <Alert kind=AlertKind::Error>
                                {format!("Failed to load: {e}")}
                            </Alert>
                        }.into_any(),
                    })}
                </Suspense>
            </div>
        </section>
    }
}

/// Calls window.loan_machine_get_access_token() and awaits its Promise.
#[cfg(target_arch = "wasm32")]
async fn get_access_token_js() -> Option<String> {
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;
    use js_sys::{Function, Promise, Reflect};

    let window = web_sys::window()?;
    let func = Reflect::get(&window, &JsValue::from_str("loan_machine_get_access_token")).ok()?;
    let func = func.dyn_into::<Function>().ok()?;
    let promise = func.call0(&JsValue::NULL).ok()?;
    let promise: Promise = promise.dyn_into().ok()?;
    let result = JsFuture::from(promise).await.ok()?;
    result.as_string()
}

#[cfg(not(target_arch = "wasm32"))]
async fn get_access_token_js() -> Option<String> {
    None
}

#[component]
fn LoadingState() -> impl IntoView {
    view! {
        <div class="flex-center" style="padding: var(--sp-16) 0">
            <span class="spinner"></span>
        </div>
    }
}

#[component]
fn CoopCard(coop: CooperativeView) -> impl IntoView {
    let (variant, badge_color, badge_text) = if coop.active {
        (CardVariant::Yellow, BadgeColor::Green, "ACTIVE")
    } else {
        (CardVariant::Default, BadgeColor::Red, "INACTIVE")
    };

    view! {
        <Card variant=variant tag="COOPERATIVE" hover=true>
            <div class="flex-between mb-4">
                <Badge color=badge_color filled=coop.active live=coop.active>
                    {badge_text}
                </Badge>
                <span class="t-mono-xs t-muted">{format!("ts {}", coop.registered_at)}</span>
            </div>
            <h3 class="t-display-md t-yellow mb-6">{coop.name.clone()}</h3>
            <div class="flex-col gap-4">
                <div class="stat-block">
                    <span class="stat-label">"Loan Machine"</span>
                    <HashDisplay value=coop.loan_machine.clone() />
                </div>
                <div class="stat-block">
                    <span class="stat-label">"Coop ID"</span>
                    <HashDisplay value=coop.coop_id.clone() />
                </div>
            </div>
        </Card>
    }
}