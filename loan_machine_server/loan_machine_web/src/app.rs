// src/app.rs
use leptos::prelude::*;
use leptos_router::components::{Router, Routes, Route};
use leptos_router::path;

use crate::components::ui::Navbar;
use crate::components::auth_bar::AuthBar;
use crate::components::gas_modal::{provide_gas_modal, GasModal};
use crate::components::gates::{
    HomeSkeleton, LoginRequiredCard, LoginRequiredKind, NotFound, RequireWallet,
};
use crate::components::home::HomePage;
use crate::components::create_coop::coop_choice::CoopChoicePage;
use crate::components::create_coop::create_coop::CreateCoopPage;
use crate::components::cooperatives::CooperativesPage;
use crate::components::vinculation::FirstVinculationForm;
use crate::wallet_auth::{provide_wallet, use_wallet, WalletSession};

// ── ROOT ──────────────────────────────────────────────────────

#[component]
pub fn App() -> impl IntoView {
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&"[app] hydrating".into());
    // Cross-cutting setup. Each call is exactly one concern.
    provide_gas_modal();
    let wallet_ctx = provide_wallet();

    // Wire Privy → WalletSession. WASM-only; SSR build skips this.
    //crate::wallet_auth::privy_bridge::install(wallet_ctx.set);

        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&"[app] calling install".into());
        #[cfg(target_arch = "wasm32")]
        crate::wallet_auth::privy_bridge::install(wallet_ctx.set);
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&"[app] install done".into());

    view! {
        <Navbar title="LOAN MACHINE" />
        <AuthBar />
        <GasModal />
        <Router>
            <Routes fallback=NotFound>
                <Route path=path!("/")             view=HomeRoute />
                <Route path=path!("/cooperatives") view=CooperativesRoute />
                <Route path=path!("/create-coop")  view=CreateCoopRoute />
                <Route path=path!("/vinculate")    view=VinculateRoute />
                <Route path=path!("/donate")       view=DonateRoute />
            </Routes>
        </Router>
    }
}

// ── ROUTES ────────────────────────────────────────────────────
// Each route is one component, one screen, one concern.

/// Home is special: it shows a real landing page when logged out,
/// not a "login required" card. So it doesn't use RequireWallet.
#[component]
fn HomeRoute() -> impl IntoView {
    let session = use_wallet().session;
    move || match session.get() {
        WalletSession::Connected { .. } => view! { <CoopChoicePage /> }.into_any(),
        WalletSession::Disconnected     => view! {
            <HomePage on_login=|| {
                #[cfg(target_arch = "wasm32")]
                crate::wallet_auth::privy_bridge::login();
            } />
        }.into_any(),
        WalletSession::Restoring        => view! { <HomeSkeleton /> }.into_any(),
    }
}


#[component]
fn CooperativesRoute() -> impl IntoView {
    view! {
        <RequireWallet
            pending=|| view! { <HomeSkeleton /> }
            fallback=|| view! { <LoginRequiredCard kind=LoginRequiredKind::Cooperatives /> }
            render=move |_| view! { <CooperativesPage /> }
        />
    }
}

#[component]
fn CreateCoopRoute() -> impl IntoView {
    view! {
        <RequireWallet
            pending=|| view! { <HomeSkeleton /> }
            fallback=|| view! { <LoginRequiredCard kind=LoginRequiredKind::CreateCoop /> }
            render=move |wallet| view! { <CreateCoopPage founder_wallet=wallet /> }
        />
    }
}

#[component]
fn VinculateRoute() -> impl IntoView {
    view! {
        <RequireWallet
            pending=|| view! { <HomeSkeleton /> }
            fallback=|| view! { <LoginRequiredCard kind=LoginRequiredKind::Vinculation /> }
            render=move |wallet| view! {
                <section class="section">
                    <div class="container-sm">
                        <FirstVinculationForm
                            smart_wallet=wallet
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
            }
        />
    }
}

#[component]
fn DonateRoute() -> impl IntoView {
    view! {
        <section class="section">
            <div class="container-sm">
                <p class="t-mono-sm t-muted">"Donate — em breve."</p>
            </div>
        </section>
    }
}