// src/components/auth_bar.rs
//
// AuthBar — wallet connection status bar
// Sits between the Navbar and page content.
//
// WHAT IT DOES:
//   - When wallet = None: explains what Privy is + shows connect button
//   - When wallet = Some: shows truncated address + logout button
//
// PROPS:
//   wallet:    ReadSignal<Option<String>>  — read from WalletRouter
//   on_login:  impl Fn()                  — called when connect clicked
//   on_logout: impl Fn()                  — called when logout clicked
//
// PLACEMENT in app.rs:
//   <Navbar />
//   <AuthBar wallet=wallet on_login=on_login on_logout=on_logout />
//   <Ticker />
//   <Router> ... </Router>

use leptos::prelude::*;
use std::sync::Arc;

#[component]
pub fn AuthBar(
    wallet:    ReadSignal<Option<String>>,
    on_login:  Arc<dyn Fn() + Send + Sync + 'static>,
    on_logout: Arc<dyn Fn() + Send + Sync + 'static>,   
) -> impl IntoView {
    view! {
        {move || {

            let login  = Arc::clone(&on_login);
            let logout = Arc::clone(&on_logout);

            match wallet.get() {
            // ── NOT CONNECTED ─────────────────────────────────
            // Show what Privy is + connect button
            None => view! {
                <div class="auth-bar auth-bar-disconnected">
                    <div class="container">
                        <div class="auth-bar-inner">

                            // Left: explanation
                            <div class="auth-bar-info">
                                <span class="auth-bar-icon">"◈"</span>
                                <div>
                                    <span class="auth-bar-label">
                                        "CARTEIRA DIGITAL — "
                                    </span>
                                    <span class="auth-bar-desc">
                                        "Privy cria uma carteira blockchain para você via e-mail ou Google. "
                                        "Sem extensão, sem seed phrase. "
                                        "Sua chave fica protegida no enclave seguro deles."
                                    </span>
                                </div>
                            </div>

                            // Right: connect button
                            <button
    class="auth-bar-btn auth-bar-btn-connect"
    on:click=move |_| {
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(
            &wasm_bindgen::JsValue::from_str("RUST: button clicked")
        );
        login()
    }
>
                                <span class="auth-bar-btn-dot auth-bar-btn-dot-off"></span>
                                "CONECTAR"
                            </button>

                        </div>
                    </div>
                </div>
            }.into_any(),

            // ── CONNECTED ─────────────────────────────────────
            // Show truncated address + logout
            Some(addr) => {
                // Truncate: 0x1234...abcd
                let display = if addr.len() > 12 {
                    format!("{}...{}", &addr[..6], &addr[addr.len()-4..])
                } else {
                    addr.clone()
                };

                view! {
                    <div class="auth-bar auth-bar-connected">
                        <div class="container">
                            <div class="auth-bar-inner">

                                // Left: connected status + address
                                <div class="auth-bar-info">
                                    <span class="auth-bar-btn-dot auth-bar-btn-dot-on"></span>
                                    <span class="auth-bar-label">"CARTEIRA CONECTADA"</span>
                                    <span class="auth-bar-address">{display}</span>
                                </div>

                                // Right: logout
                                <button
                                    class="auth-bar-btn auth-bar-btn-logout"
                                    on:click=move |_| logout()
                                >
                                    "DESCONECTAR"
                                </button>

                            </div>
                        </div>
                    </div>
                }.into_any()
            }
            }
        }}
    }
}