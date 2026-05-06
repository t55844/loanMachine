// src/app.rs
use leptos::prelude::*;
use leptos_router::components::{Router, Routes, Route};
use leptos_router::path;
use crate::components::ui::*;
use crate::components::vinculation::{VinculationGate, FirstVinculationForm};
use crate::components::auth_bar::AuthBar;
use crate::components::home::HomePage;
use crate::components::create_coop::coop_choice::CoopChoicePage;
use crate::components::create_coop::create_coop::CreateCoopPage;
use crate::components::gas_modal::{provide_gas_modal, GasModal};
use crate::components::cooperatives::{CooperativesPage};
use std::sync::Arc;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
// ── ROOT ──────────────────────────────────────────────────────

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Navbar title="LOAN MACHINE" />
        <WalletRouter />
    }
}

// ── WALLET ROUTER ─────────────────────────────────────────────
// Owns the single source of truth for wallet state.
// Passes it down to AuthBar (display) and Routes (gating).

#[component]
fn WalletRouter() -> impl IntoView {

    provide_gas_modal();
    
    let (wallet, set_wallet) = signal::<Option<String>>(None);

    // Browser-only setup: register listeners + trigger session restore.
    // Runs once per hydration, directly in the component body — no Effect needed.
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::prelude::*;

        let window = web_sys::window().expect("no window");

        // ── privy_wallet_ready → set wallet ──
        let on_ready = Closure::<dyn Fn(web_sys::CustomEvent)>::new(
            move |e: web_sys::CustomEvent| {
                let address = js_sys::Reflect::get(&e.detail(), &JsValue::from_str("address"))
                    .ok()
                    .and_then(|v| v.as_string());
                if let Some(addr) = address {
                    web_sys::console::log_1(
                        &JsValue::from_str(&format!("[rust] privy_wallet_ready received: {addr}"))
                    );
                    set_wallet.set(Some(addr));
                }
            },
        );
        window
            .add_event_listener_with_callback(
                "privy_wallet_ready",
                on_ready.as_ref().unchecked_ref(),
            )
            .expect("add privy_wallet_ready listener");
        on_ready.forget(); // keep alive for page lifetime

        // ── privy_logged_out → clear wallet ──
        let on_logout = Closure::<dyn Fn(web_sys::CustomEvent)>::new(
            move |_e: web_sys::CustomEvent| {
                web_sys::console::log_1(&JsValue::from_str("[rust] privy_logged_out received"));
                set_wallet.set(None);
            },
        );
        window
            .add_event_listener_with_callback(
                "privy_logged_out",
                on_logout.as_ref().unchecked_ref(),
            )
            .expect("add privy_logged_out listener");
        on_logout.forget();

        // ── Trigger restore (listeners are armed) ──
        web_sys::console::log_1(&JsValue::from_str("[rust] calling loan_machine_try_restore"));
        call_window_fn_when_ready("loan_machine_try_restore", 30, 100);

    }

    // ── Login / logout closures ──
    let login_fn: Arc<dyn Fn() + Send + Sync> = Arc::new(|| {
        #[cfg(target_arch = "wasm32")]
        call_window_fn("loan_machine_init_privy");
    });

    let logout_fn: Arc<dyn Fn() + Send + Sync> = Arc::new(|| {
        #[cfg(target_arch = "wasm32")]
        call_window_fn("loan_machine_logout");
    });

    view! {
        <AuthBar
            wallet=wallet
            on_login=Arc::clone(&login_fn)
            on_logout=Arc::clone(&logout_fn)
        />
        <GasModal />
        <Router>
            <Routes fallback=|| view! {
                <section class="section">
                    <div class="container-sm">
                        <Alert kind=AlertKind::Error>"Página não encontrada."</Alert>
                    </div>
                </section>
            }>
                <Route path=path!("/") view=move || {
                    let login = Arc::clone(&login_fn);
                    match wallet.get() {
                        None => view! {
                            <HomePage on_login=move || login() />
                        }.into_any(),
                        Some(addr) => view! { <CoopChoicePage /> }.into_any(),
                    }
                }/>

                <Route path=path!("/cooperatives") view=move || {
                    match wallet.get() {
                        None => view! {
                            <section class="section">
                                <div class="container-sm">
                                    <Card variant=CardVariant::Yellow hover=false>
                                        <h2 class="t-display-md t-yellow">"LOGIN NECESSÁRIO"</h2>
                                        <p class="t-mono-sm t-muted mt-4">
                                            "Conecte sua carteira para ver as cooperativas."
                                        </p>
                                    </Card>
                                </div>
                            </section>
                        }.into_any(),
                        Some(_) => view! {
                            <CooperativesPage wallet=wallet />
                        }.into_any(),
                    }
                } />

                <Route path=path!("/create-coop") view=move || {
                    match wallet.get() {
                        None => view! {  
                            <div class="auth-bar-info">
                                    <span class="auth-bar-btn-dot auth-bar-btn-dot-on"></span>
                                    <span class="auth-bar-label">"CARTEIRA NÃO CONECTADA"</span>
                                </div> }.into_any(),
                        Some(addr) => view! { <CreateCoopPage founder_wallet=addr /> }.into_any(),
                    }
                } />

                <Route path=path!("/vinculate") view=move || {
                    match wallet.get() {
                        None => view! {
                            <section class="section">
                                <div class="container-sm">
                                    <Card variant=CardVariant::Yellow hover=false>
                                        <div class="flex-col gap-6">
                                            <h2 class="t-display-md t-yellow">"LOGIN NECESSÁRIO"</h2>
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

#[cfg(target_arch = "wasm32")]
fn call_window_fn(name: &str) {
    use js_sys::Function;
    let window = web_sys::window().expect("window");
    if let Ok(val) = js_sys::Reflect::get(&window, &JsValue::from_str(name)) {
        if let Ok(func) = val.dyn_into::<Function>() {
            let _ = func.call0(&JsValue::NULL);
            return;
        }
    }
    web_sys::console::error_1(
        &JsValue::from_str(&format!("[rust] {name} not found / not a function"))
    );
}

#[cfg(target_arch = "wasm32")]
fn call_window_fn_when_ready(name: &'static str, max_attempts: u32, interval_ms: i32) {
    use js_sys::Function;

    fn try_now(name: &str) -> bool {
        let window = match web_sys::window() {
            Some(w) => w,
            None    => return false,
        };
        if let Ok(val) = js_sys::Reflect::get(&window, &JsValue::from_str(name)) {
            if let Ok(func) = val.dyn_into::<Function>() {
                let _ = func.call0(&JsValue::NULL);
                return true;
            }
        }
        false
    }

    if try_now(name) {
        return;
    }

    let attempts = std::rc::Rc::new(std::cell::Cell::new(0u32));
    // We need a Closure that can re-schedule itself, which requires
    // the Closure to be reachable from inside its own body. The
    // Rc<RefCell<Option<Closure>>> dance below is the standard pattern.
    let cb_holder: std::rc::Rc<std::cell::RefCell<Option<Closure<dyn FnMut()>>>> =
        std::rc::Rc::new(std::cell::RefCell::new(None));

    let cb_holder_clone = cb_holder.clone();
    let attempts_clone  = attempts.clone();

    let cb = Closure::<dyn FnMut()>::new(move || {
        if try_now(name) {
            // Done — drop the closure so it gets cleaned up.
            cb_holder_clone.borrow_mut().take();
            return;
        }

        let n = attempts_clone.get() + 1;
        attempts_clone.set(n);

        if n >= max_attempts {
            web_sys::console::error_1(
                &JsValue::from_str(&format!("[rust] {name} never became available after {n} attempts"))
            );
            cb_holder_clone.borrow_mut().take();
            return;
        }

        // Re-schedule.
        let window = web_sys::window().expect("window");
        if let Some(cb) = cb_holder_clone.borrow().as_ref() {
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                interval_ms,
            );
        }
    });

    // Schedule the first poll.
    let window = web_sys::window().expect("window");
    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        cb.as_ref().unchecked_ref(),
        interval_ms,
    );

    // Store the closure so it stays alive across timer ticks.
    *cb_holder.borrow_mut() = Some(cb);
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