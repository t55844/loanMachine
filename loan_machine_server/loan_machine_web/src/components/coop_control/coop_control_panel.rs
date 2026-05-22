// loan_machine_web/src/components/coop_control_panel.rs

use leptos::prelude::*;
use leptos::server_fn::ServerFnError;
use leptos_router::hooks::use_params;
use leptos_router::params::Params;

use crate::components::gates::{
    HomeSkeleton, LoginRequiredCard, LoginRequiredKind, RequireWallet,
};
use crate::components::ui::*;
use crate::server_fns::cooperatives::get_coop_viewer_state;
use crate::wallet_auth::privy_bridge;
use loan_machine_models::responses::{CooperativeView, ViewerRole};
use loan_machine_models::wallet_address::WalletAddress;

// ── PARAMS ──────────────────────────────────────────────────

#[derive(Params, PartialEq, Clone)]
struct CoopParams {
    id: Option<String>,
}

// ── ROUTE ENTRY ─────────────────────────────────────────────

#[component]
pub fn CoopControlPanelRoute() -> impl IntoView {
    view! {
        <RequireWallet
            pending=|| view! { <HomeSkeleton /> }
            fallback=|| view! { <LoginRequiredCard kind=LoginRequiredKind::Cooperatives /> }
            render=move |wallet| view! { <CoopControlPanel wallet=wallet /> }
        />
    }
}

// ── PANEL ───────────────────────────────────────────────────

#[component]
pub(crate) fn CoopControlPanel(wallet: WalletAddress) -> impl IntoView {
    let params = use_params::<CoopParams>();

    // Bumped by sub-components after a successful tx so the role
    // re-resolves (Approved → Member, etc.) without a page reload.
    let (refresh, set_refresh) = signal(0u64);

    let coop_id = Memo::new(move |_| {
        params.read().as_ref().ok().and_then(|p| p.id.clone())
    });

    let state = LocalResource::new(move || {
        let id = coop_id.get();
        let _tick = refresh.get(); // subscribe so bumps refetch
        async move {
            let Some(id) = id else {
                return Err(ServerFnError::new("missing_coop_id"));
            };
            let token = privy_bridge::get_access_token().await.unwrap_or_default();
            if token.is_empty() {
                return Err(ServerFnError::new("no_token_available"));
            }
            get_coop_viewer_state(id).await
        }
    });

    let on_state_changed = Callback::new(move |()| set_refresh.update(|n| *n += 1));

    view! {
        <section class="section">
            <div class="container">
                <Suspense fallback=|| view! { <PanelSkeleton /> }>
                    {move || state.get().map(|res| match res {
                        Err(e) => view! {
                            <Alert kind=AlertKind::Error>
                                {format!("Failed to load coop: {e}")}
                            </Alert>
                        }.into_any(),
                        Ok(s) => view! {
                            <CoopHeader coop=s.coop.clone() />
                            <Divider />
                            <RoleSection
                                role=s.role
                                is_admin=s.is_admin
                                is_moderator=s.is_moderator
                                wallet=wallet.clone()
                                coop=s.coop
                                on_state_changed=on_state_changed
                            />
                        }.into_any(),
                    })}
                </Suspense>
            </div>
        </section>
    }
}

#[component]
fn PanelSkeleton() -> impl IntoView {
    view! {
        <div class="flex-center" style="padding: var(--sp-16) 0">
            <span class="spinner"></span>
        </div>
    }
}

// ── HEADER (always shown) ───────────────────────────────────

#[component]
pub(crate) fn CoopHeader(coop: CooperativeView) -> impl IntoView {
    let (badge_color, badge_text) = if coop.active {
        (BadgeColor::Green, "ACTIVE")
    } else {
        (BadgeColor::Red, "INACTIVE")
    };
    let variant = if coop.active { CardVariant::Yellow } else { CardVariant::Default };

    view! {
        <Card variant=variant tag="COOPERATIVE" hover=false>
            <div class="flex-between mb-4">
                <Badge color=badge_color filled=coop.active live=coop.active>
                    {badge_text}
                </Badge>
                <span class="t-mono-xs t-muted">
                    {format!("ts {}", coop.registered_at)}
                </span>
            </div>
            <h2 class="t-display-md t-yellow mb-6">{coop.name.clone()}</h2>
            <div class="flex-col gap-4">
                <div class="stat-block">
                    <DataLabel>"Loan Machine"</DataLabel>
                    <HashDisplay value=coop.loan_machine.clone() />
                </div>
                <div class="stat-block">
                    <DataLabel>"Coop ID"</DataLabel>
                    <HashDisplay value=coop.coop_id.clone() />
                </div>
            </div>
        </Card>
    }
}

// ── DISPATCH (the panel's only real job) ────────────────────
use crate::components::vinculation::FirstVinculationForm;
use crate::components::coop_control::approval_pending::ApprovalPending;
use crate::components::coop_control::coop_approval::RequestApproval;
use crate::components::coop_control::admin_panel::{AdminPanel, ModeratorPanel};
#[component]
pub(crate) fn RoleSection(
    role: ViewerRole,
    is_admin: bool,
    is_moderator: bool,
    wallet: WalletAddress,
    coop: CooperativeView,
    on_state_changed: Callback<()>,
) -> impl IntoView {
    let _ = &wallet;
    let coop_id = coop.coop_id.clone();

    // 1. Membership ladder — what's the user's relationship to the coop?
    let ladder = match role {
        ViewerRole::Visitor => view! {
            <RequestApproval coop_id=coop_id.clone() on_tx_success=on_state_changed />
        }.into_any(),

        ViewerRole::ApprovalPending => view! {
            <ApprovalPending coop_id=coop_id.clone() />
        }.into_any(),

        ViewerRole::Approved => view! {
            <FirstVinculationForm on_success=move || on_state_changed.run(()) />
        }.into_any(),

        ViewerRole::Member => view! {
            <TodoPlaceholder label="MEMBER PANEL" />
        }.into_any(),

        // Legacy variants — kept only if you haven't deleted them yet.
        ViewerRole::Admin | ViewerRole::Moderator => view! {
            <TodoPlaceholder label="MEMBER PANEL" />
        }.into_any(),
    };

    // 2. Capability panels — additive. Admin and moderator can both render.
    view! {
        {ladder}
        {is_admin.then(|| view! {
            <div style="margin-top: var(--sp-6)">
                <AdminPanel coop_id=coop_id.clone() on_tx_success=on_state_changed />
            </div>
        })}
        {is_moderator.then(|| view! {
            <div style="margin-top: var(--sp-6)">
                <ModeratorPanel coop_id=coop_id.clone() on_tx_success=on_state_changed />
            </div>
        })}
    }
}

/// Honest "this isn't built yet" placeholder.  Every role gets one
/// until its real sub-component lands.  Visible in dev, easy to grep.
#[component]
fn TodoPlaceholder(label: &'static str) -> impl IntoView {
    view! {
        <Card variant=CardVariant::Default hover=false>
            <Badge color=BadgeColor::Gold>"TODO"</Badge>
            <p class="t-mono-sm t-muted" style="margin-top: var(--sp-4)">
                {format!("Sub-component pending: {label}")}
            </p>
        </Card>
    }
}