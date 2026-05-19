// src/app.rs
use leptos::prelude::*;
use leptos_router::components::{Router, Routes, Route};
use leptos_router::path;

use crate::components::auth_bar::AuthBar;
use crate::components::gas_modal::{provide_gas_modal, GasModal};
use crate::components::gates::{
    HomeSkeleton, LoginRequiredCard, LoginRequiredKind, NotFound, RequireWallet,
};
use crate::components::home::HomePage;
use crate::components::create_coop::coop_choice::CoopChoicePage;
use crate::components::create_coop::create_coop::CreateCoopPage;
use crate::components::cooperatives::CooperativesPage;
use crate::wallet_auth::{provide_wallet, use_wallet, WalletSession};
use crate::components::coop_control::coop_control_panel::CoopControlPanelRoute;
use crate::components::user::user_coops::{UserCoopsPage};
use crate::components::elections::elections::ElectionsPage;
use crate::components::vinculation::FirstVinculationForm;
use crate::components::helpers::prices::{provide_prices_context};


// ── NAVIGATION ──────────────────────────────────────────────

#[component]
pub fn Navbar(
    #[prop(optional, default="LOAN MACHINE")] title: &'static str,
) -> impl IntoView {
    view! {
        <nav class="navbar">
            <div class="container navbar-inner">
                <a href="/" class="navbar-brand">{title}</a>
                <ul class="navbar-links">
                    <li><a href="/" class="navbar-link">"Pagina inicial"</a></li>
                    <li><a href="/cooperatives" class="navbar-link">"Cooperativas"</a></li>
                    <li><a href="/create-coop" class="navbar-link">"Crie uma Cooper."</a></li>
                    <li><a href="/user" class="navbar-link">"Usuario"</a></li>
                    <li>
                        <span class="badge badge-live badge-yellow">"Live"</span>
                    </li>
                </ul>
            </div>
        </nav>
    }
}

// ── ROOT ──────────────────────────────────────────────────────

#[component]
pub fn App() -> impl IntoView {
    // Cross-cutting setup. Each call is exactly one concern.
    provide_prices_context();
    provide_gas_modal();
    let wallet_ctx = provide_wallet();

    // Wire Privy → WalletSession. WASM-only; SSR build skips this.
    //crate::wallet_auth::privy_bridge::install(wallet_ctx.set);
    crate::wallet_auth::privy_bridge::install(wallet_ctx.set);


    view! {
        <Navbar title="LOAN MACHINE" />
        <AuthBar />
        <GasModal />
        <Router>
            <Routes fallback=NotFound>
                <Route path=path!("/")             view=HomeRoute />
                <Route path=path!("/cooperatives") view=CooperativesRoute />
                <Route path=path!("/cooperatives/:id") view=CoopControlPanelRoute />
                <Route path=path!("/create-coop")  view=CreateCoopRoute />
                <Route path=path!("/user")    view=UserRoute />

                <Route path=path!("/elections/:id")    view=ElectionsRoute/>
                <Route path=path!("/vinculation")    view=VinculateRoute />
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
fn UserRoute() -> impl IntoView {
    view! {
        <RequireWallet
            pending=|| view! { <HomeSkeleton /> }
            fallback=|| view! { <LoginRequiredCard kind=LoginRequiredKind::CreateCoop /> }
            render=move |_| view! { <UserCoopsPage  /> }
        />
    }
}

#[component]
fn ElectionsRoute() -> impl IntoView {
    use leptos_router::hooks::use_params_map;
    let params = use_params_map();
    view! {
        <RequireWallet
            pending=|| view! { <HomeSkeleton /> }
            fallback=|| view! { <LoginRequiredCard kind=LoginRequiredKind::CreateCoop /> }
            render=move |wallet| {
                let coop_id = params.read().get("id").unwrap_or_default();
                view! { <ElectionsPage coop_id caller_wallet=wallet /> }
            }
        />
    }
}


#[component]
fn VinculateRoute() -> impl IntoView {
    use leptos_router::hooks::use_navigate;
    let navigate = use_navigate();
    view! {
        <RequireWallet
            pending=|| view! { <HomeSkeleton /> }
            fallback=|| view! { <LoginRequiredCard kind=LoginRequiredKind::CreateCoop /> }
            render=move |_wallet| {
                let nav = navigate.clone();
                view! {
                    <FirstVinculationForm on_success=move || {
                        nav("/user", Default::default());
                    } />
                }
            }
        />
    }
}