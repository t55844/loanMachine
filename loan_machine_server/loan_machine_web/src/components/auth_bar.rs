// loan_machine_web/src/components/auth_bar.rs

use leptos::prelude::*;

use crate::server_fns::wallet_balance::get_wallet_balance_wei;
use crate::components::helpers::prices::{ use_prices};
use crate::wallet_auth::{use_wallet, WalletSession};
use crate::wallet_auth::privy_bridge::{self, TxOutcome};
use loan_machine_models::wallet_address::WalletAddress;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[component]
pub fn AuthBar() -> impl IntoView {

    let session = use_wallet().session;

    let root_class = move || match session.get() {
        WalletSession::Connected { .. } => "auth-bar auth-bar-connected",
        _ => "auth-bar auth-bar-disconnected",
    };

    view! {
        <div class=root_class>
            <div class="container auth-bar-inner">
                {move || match session.get() {
                    WalletSession::Connected { wallet } => view! {
                        <ConnectedBar wallet=wallet />
                    }.into_any(),

                    WalletSession::Disconnected => view! {
                        <DisconnectedBar />
                    }.into_any(),

                    WalletSession::Restoring => view! {
                        <div class="auth-bar-info">
                            <span class="auth-bar-btn-dot auth-bar-btn-dot-pending"></span>
                            <span class="auth-bar-label">"RESTORING…"</span>
                        </div>
                    }.into_any(),
                }}
            </div>
        </div>
    }
}

// ── Connected ──────────────────────────────────────────────

#[component]
fn ConnectedBar(wallet: WalletAddress) -> impl IntoView {
    let wallet_str = wallet.to_string();
    let short      = wallet.short();

    // Refresh balance after every tx outcome.
    let (tick, set_tick) = signal(0u32);
    privy_bridge::on_tx_outcome(move |out| {
        if matches!(out, TxOutcome::Complete(_)) {
            set_tick.update(|n| *n += 1);
        }
    });

    let balance = LocalResource::new(move || {
        let _ = tick.get();
        async move { get_wallet_balance_wei().await.ok() }
    });

    view! {
        <div class="auth-bar-info">
            <span class="auth-bar-btn-dot auth-bar-btn-dot-on"></span>
            <span class="auth-bar-label">"WALLET CONNECTED"</span>
            <WalletWithCopy short=short.clone() full=wallet_str.clone() />
            <BalanceRow balance=balance />
        </div>
        <button
            class="auth-bar-btn auth-bar-btn-logout"
            on:click=|_| {
                #[cfg(target_arch = "wasm32")]
                crate::wallet_auth::privy_bridge::logout();
            }
        >
            "DISCONNECT"
        </button>
    }
}
use crate::components::ui::{CopyButton, Money, MoneyCurrency};

#[component]
fn WalletWithCopy(short: String, full: String) -> impl IntoView {
    view! {
        <span class="auth-bar-address">{short}</span>
        <CopyButton value=full title="Copy full address" />
    }
}

#[component]
fn BalanceRow(balance: LocalResource<Option<String>>) -> impl IntoView {
    let prices = use_prices();

    let view_fn = move || {
        let Some(wei_str) = balance.get().flatten() else {
            return view! {
                <span class="auth-bar-balance t-muted">"balance…"</span>
            }.into_any();
        };

        let eth = wei_str.parse::<f64>().unwrap_or(0.0) / 1e18;
        let usd = prices.get().map(|p| eth * p.eth_usd);

        view! {
            <span class="auth-bar-balance">
                <Money currency=MoneyCurrency::Eth value=Some(eth) />
                <Money currency=MoneyCurrency::Usd value=usd />
            </span>
        }.into_any()
    };

    view! { {view_fn} }
}

// ── Disconnected (unchanged) ───────────────────────────────

#[component]
fn DisconnectedBar() -> impl IntoView {
    view! {
        <div class="auth-bar-info">
            <span class="auth-bar-btn-dot auth-bar-btn-dot-off"></span>
            <span class="auth-bar-icon">"◈"</span>
            <span class="auth-bar-label">"DIGITAL WALLET"</span>
            <span class="auth-bar-desc">
                "Privy creates a blockchain wallet for you via email or Google. "
                "No extension, no seed phrase. "
                "Your key is protected in their secure enclave."
            </span>
        </div>
        <button
            class="auth-bar-btn auth-bar-btn-connect"
            on:click=|_| {
                #[cfg(target_arch = "wasm32")]
                crate::wallet_auth::privy_bridge::login();
            }
        >
            "CONNECT"
        </button>
    }
}