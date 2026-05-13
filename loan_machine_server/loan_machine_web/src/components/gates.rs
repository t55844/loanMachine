// src/components/gates.rs
use leptos::prelude::*;
use crate::components::ui::{Alert, AlertKind, Card, CardVariant};
use crate::wallet_auth::{use_wallet, WalletSession};
use loan_machine_models::wallet_address::WalletAddress;
// ── DECISION (pure, testable) ─────────────────────────────────

/// Which branch of the gate to render for a given session + prop config.
/// Pure function — no view types, no reactive context.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum GateOutcome<'a> {
    /// Restoring, no `pending` view provided → render nothing.
    Skeleton,
    /// Restoring, `pending` view provided → render it.
    Pending,
    /// Disconnected → render `fallback`.
    Fallback,
    /// Connected → call `render(wallet)`.
    Render(&'a WalletAddress),
}

pub(crate) fn outcome<'a>(session: &'a WalletSession, has_pending: bool) -> GateOutcome<'a> {
    match session {
        WalletSession::Restoring if has_pending => GateOutcome::Pending,
        WalletSession::Restoring                => GateOutcome::Skeleton,
        WalletSession::Disconnected             => GateOutcome::Fallback,
        WalletSession::Connected { wallet }     => GateOutcome::Render(wallet),
    }
}

// ── GATE COMPONENT ────────────────────────────────────────────

#[component]
pub fn RequireWallet<F, IV>(
    #[prop(optional, into)] pending: Option<ViewFn>,
    #[prop(into)]           fallback: ViewFn,
    render: F,
) -> impl IntoView
where
    F: Fn(WalletAddress) -> IV + Send + Sync + 'static,
    IV: IntoView + 'static,
{
    let session = use_wallet().session;
    move || {
        let s = session.get();
        match outcome(&s, pending.is_some()) {
            GateOutcome::Pending      => pending.as_ref().unwrap().run().into_any(),
            GateOutcome::Skeleton     => ().into_any(),
            GateOutcome::Fallback     => fallback.run().into_any(),
            GateOutcome::Render(addr) => render(addr.clone()).into_any(),
        }
    }
}

// ── LOGIN REQUIRED CARD ───────────────────────────────────────

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LoginRequiredKind {
    Cooperatives,
    Vinculation,
    CreateCoop,
}

impl LoginRequiredKind {
    pub fn body_text(self) -> &'static str {
        match self {
            Self::Cooperatives => "Conecte sua carteira para ver as cooperativas.",
            Self::Vinculation  => "Use o botão CONECTAR acima para criar ou acessar sua carteira Privy.",
            Self::CreateCoop   => "Conecte sua carteira para criar uma cooperativa.",
        }
    }
}

#[component]
pub fn LoginRequiredCard(kind: LoginRequiredKind) -> impl IntoView {
    let body = kind.body_text();
    view! {
        <section class="section">
            <div class="container-sm">
                <Card variant=CardVariant::Yellow hover=false>
                    <h2 class="t-display-md t-yellow">"LOGIN NECESSÁRIO"</h2>
                    <p class="t-mono-sm t-muted" style="margin-top: var(--sp-4)">{body}</p>
                </Card>
            </div>
        </section>
    }
}

// ── PLACEHOLDERS ──────────────────────────────────────────────

#[component]
pub fn HomeSkeleton() -> impl IntoView {
    view! {
        <section class="section">
            <div class="container-sm">
                <Card variant=CardVariant::Default hover=false>
                    <div class="flex-col gap-6" style="opacity: 0.4">
                        <div class="t-display-md">"…"</div>
                        <p class="t-mono-sm t-muted">"Carregando sessão…"</p>
                    </div>
                </Card>
            </div>
        </section>
    }
}

#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <section class="section">
            <div class="container-sm">
                <Alert kind=AlertKind::Error>"Página não encontrada."</Alert>
            </div>
        </section>
    }
}