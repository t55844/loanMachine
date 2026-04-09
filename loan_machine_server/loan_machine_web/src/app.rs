// src/app.rs
use leptos::prelude::*;
use leptos_router::components::{Router, Routes, Route};
use leptos_router::path;
use crate::components::ui::*;
use crate::components::vinculation::{VinculationGate, FirstVinculationForm};
use crate::components::auth_bar::AuthBar;
use crate::components::home::HomePage;
use std::sync::Arc;

// ── ROOT ──────────────────────────────────────────────────────

#[component]
pub fn App() -> impl IntoView {
    view! {       
        <Navbar title="LOAN MACHINE" />
        // WalletRouter owns the wallet signal and passes it down
        // It must wrap both AuthBar and Router so both share the same signal
        <WalletRouter />
    }
}

// ── WALLET ROUTER ─────────────────────────────────────────────
// Owns the single source of truth for wallet state.
// Passes it down to AuthBar (display) and Routes (gating).

#[component]
fn WalletRouter() -> impl IntoView {
    let (wallet, set_wallet) = signal::<Option<String>>(None);

    // ── Listen for privy_wallet_ready → set wallet ────────────
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::prelude::*;
        use wasm_bindgen::JsCast;

        let set_w = set_wallet.clone();
        let closure = Closure::<dyn Fn(web_sys::CustomEvent)>::new(
            move |e: web_sys::CustomEvent| {
                let address = js_sys::Reflect::get(
                    &e.detail(),
                    &JsValue::from_str("address"),
                )
                .ok()
                .and_then(|v: JsValue| v.as_string());

                if let Some(addr) = address {
                    set_w.set(Some(addr));
                }
            },
        );
        web_sys::window()
            .expect("no window")
            .add_event_listener_with_callback(
                "privy_wallet_ready",
                closure.as_ref().unchecked_ref(),
            )
            .expect("failed to add event listener");
        closure.forget();
    }

    // ── Listen for privy_logged_out → clear wallet ────────────
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::prelude::*;
        use wasm_bindgen::JsCast;

        let set_w = set_wallet.clone();
        let closure = Closure::<dyn Fn(web_sys::CustomEvent)>::new(
            move |_e: web_sys::CustomEvent| {
                set_w.set(None);
            },
        );
        web_sys::window()
            .expect("no window")
            .add_event_listener_with_callback(
                "privy_logged_out",
                closure.as_ref().unchecked_ref(),
            )
            .expect("failed to add event listener");
        closure.forget();
    }

    // Helper closures — defined once, reused across routes
    // WHY not inline: the wasm_bindgen extern block can't be written
    // inside a move closure cleanly — extract to a plain fn call
    let login_fn: Arc<dyn Fn() + Send + Sync> = Arc::new(|| {
            #[cfg(target_arch = "wasm32")]
            {
                use wasm_bindgen::JsValue;
                use js_sys::Function;
                use wasm_bindgen::JsCast;

                let window = web_sys::window().unwrap();
                // Look up the function on window at runtime — not compile-time binding
                match js_sys::Reflect::get(&window, &JsValue::from_str("loan_machine_init_privy")) {
                    Ok(val) => {
                        if let Ok(func) = val.dyn_into::<Function>() {
                            let _ = func.call0(&JsValue::NULL);
                        } else {
                            web_sys::console::error_1(
                                &JsValue::from_str("loan_machine_init_privy is not a function — bridge not loaded?")
                            );
                        }
                    }
                    Err(_) => {
                        web_sys::console::error_1(
                            &JsValue::from_str("loan_machine_init_privy not found on window — bridge not loaded?")
                        );
                    }
                }
            }
        });

        let logout_fn: Arc<dyn Fn() + Send + Sync> = Arc::new(|| {
            #[cfg(target_arch = "wasm32")]
            {
                use wasm_bindgen::JsValue;
                use js_sys::Function;
                use wasm_bindgen::JsCast;

                let window = web_sys::window().unwrap();
                match js_sys::Reflect::get(&window, &JsValue::from_str("loan_machine_logout")) {
                    Ok(val) => {
                        if let Ok(func) = val.dyn_into::<Function>() {
                            let _ = func.call0(&JsValue::NULL);
                        }
                    }
                    Err(_) => {}
                }
            }
        });

    view! {
        // AuthBar: always visible, shows connect or address
        <AuthBar
            wallet=wallet
            on_login=Arc::clone(&login_fn)  
            on_logout=Arc::clone(&logout_fn)
        />

        <Router>
            <Routes fallback=|| view! {
                <section class="section">
                    <div class="container-sm">
                        <Alert kind=AlertKind::Error>
                            "Página não encontrada."
                        </Alert>
                    </div>
                </section>
            }>

                // ── / ─────────────────────────────────────────
                <Route path=path!("/") view=move || {
                let login = Arc::clone(&login_fn);  
                    match wallet.get() {
                        None => view! {
                            <HomePage on_login=move || login() />
                        }.into_any(),
                        Some(addr) => view! {
                            <VinculationGate smart_wallet=addr>
                                <MainDashboard />
                            </VinculationGate>
                        }.into_any(),
                    }
                }/>

                // ── /vinculate ────────────────────────────────
                <Route path=path!("/vinculate") view=move || {
                    match wallet.get() {
                        // Not logged in: point to the AuthBar above
                        None => view! {
                            <section class="section">
                                <div class="container-sm">
                                    <Card variant=CardVariant::Yellow hover=false>
                                        <div class="flex-col gap-6">
                                            <h2 class="t-display-md t-yellow">
                                                "LOGIN NECESSÁRIO"
                                            </h2>
                                            <p class="t-mono-sm t-muted">
                                                "Use o botão "
                                                <span class="t-yellow">"CONECTAR"</span>
                                                " acima para criar ou acessar sua carteira Privy."
                                            </p>
                                        </div>
                                    </Card>
                                </div>
                            </section>
                        }.into_any(),

                        // Logged in: show form directly
                        Some(addr) => view! {
                            <section class="section">
                                <div class="container-sm">
                                    <FirstVinculationForm
                                        smart_wallet=addr
                                        on_success=move |_coop| {
                                            #[cfg(target_arch = "wasm32")]
                                            {
                                                let _ = web_sys::window()
                                                    .unwrap()
                                                    .location()
                                                    .set_href("/");
                                            }
                                        }
                                    />
                                </div>
                            </section>
                        }.into_any(),
                    }
                }/>

                // ── /donate ───────────────────────────────────
                <Route path=path!("/donate") view=|| view! {
                    <section class="section">
                        <div class="container-sm">
                            <p class="t-mono-sm t-muted">"Donate — em breve."</p>
                        </div>
                    </section>
                }/>

            </Routes>
        </Router>
    }
}

// ── MAIN DASHBOARD ────────────────────────────────────────────

#[component]
fn MainDashboard() -> impl IntoView {
    view! {
        <section class="section">
            <div class="container">
                <div class="t-cordel-rule t-display-md t-yellow mb-8">"DASHBOARD"</div>
                <div class="grid-3">
                    <Card variant=CardVariant::Yellow hover=true>
                        <div class="stat-block">
                            <span class="stat-label">"Saldo"</span>
                            <span class="stat-value">"R$ 0"</span>
                        </div>
                    </Card>
                    <Card variant=CardVariant::Default hover=true>
                        <div class="stat-block">
                            <span class="stat-label">"Empréstimos Ativos"</span>
                            <span class="stat-value">"0"</span>
                        </div>
                    </Card>
                    <Card variant=CardVariant::Gold hover=true>
                        <div class="stat-block">
                            <span class="stat-label">"Reputação"</span>
                            <span class="stat-value">"0"</span>
                        </div>
                    </Card>
                </div>
            </div>
        </section>
    }
}