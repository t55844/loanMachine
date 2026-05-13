// src/components/auth_bar.rs

use leptos::prelude::*;
use crate::wallet_auth::{use_wallet, WalletSession};

#[component]
pub fn AuthBar() -> impl IntoView {
    let session = use_wallet().session;

    // Root class flips on state so the CSS theme switches between yellow/green.
    let root_class = move || match session.get() {
        WalletSession::Connected { .. } => "auth-bar auth-bar-connected",
        _ => "auth-bar auth-bar-disconnected",
    };

    view! {
        <div class=root_class>
            <div class="container auth-bar-inner">
                {move || match session.get() {
                    WalletSession::Connected { wallet } => view! {
                        <div class="auth-bar-info">
                            <span class="auth-bar-btn-dot auth-bar-btn-dot-on"></span>
                            <span class="auth-bar-label">"CARTEIRA CONECTADA"</span>
                            <span class="auth-bar-address">{wallet.short()}</span>
                        </div>
                        <button
                            class="auth-bar-btn auth-bar-btn-logout"
                            on:click=|_| {
                                #[cfg(target_arch = "wasm32")]
                                crate::wallet_auth::privy_bridge::logout();
                            }
                        >
                            "DESCONECTAR"
                        </button>
                    }.into_any(),

                    WalletSession::Disconnected => view! {
                        <div class="auth-bar-info">
                            <span class="auth-bar-btn-dot auth-bar-btn-dot-off"></span>
                            <span class="auth-bar-icon">"◈"</span>
                            <span class="auth-bar-label">"CARTEIRA DIGITAL"</span>
                            <span class="auth-bar-desc">
                                "Privy cria uma carteira blockchain para você via e-mail ou Google. "
                                "Sem extensão, sem seed phrase. "
                                "Sua chave fica protegida no enclave seguro deles."
                            </span>
                        </div>
                        <button
                            class="auth-bar-btn auth-bar-btn-connect"
                            on:click=|_| {
                                #[cfg(target_arch = "wasm32")]
                                crate::wallet_auth::privy_bridge::login();
                            }
                        >
                            "CONECTAR"
                        </button>
                    }.into_any(),

                    WalletSession::Restoring => view! {
                        <div class="auth-bar-info">
                            <span class="auth-bar-btn-dot auth-bar-btn-dot-pending"></span>
                            <span class="auth-bar-label">"RESTAURANDO…"</span>
                        </div>
                    }.into_any(),
                }}
            </div>
        </div>
    }
}