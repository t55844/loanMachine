// loan_machine_web/src/components/user_coops.rs
//
// "My cooperatives" — the user-status panel.
//
// Single Resource → list of UserCoop.  Each entry renders as a Card with
// the coop's metadata and a row of action links (currently just Elections;
// loans, donations, withdrawals get added here as those flows ship).
//
// Auth: get_user_coops() reads the wallet from the JWT server-side.
// The page itself is gated by RequireWallet in app.rs; nothing here
// needs the wallet as a prop.

use leptos::prelude::*;

use loan_machine_models::responses::UserCoop;

use crate::components::ui::*;
use crate::server_fns::user_coops::get_user_coops;

#[component]
pub fn UserCoopsPage() -> impl IntoView {
    let coops = Resource::new(
        || (),
        |_| async move { get_user_coops().await },
    );

    view! {
        <Section>
            <div class="container-md">
                <div class="t-center" style="margin-bottom: var(--sp-8)">
                    <SectionTitle>"MINHAS COOPERATIVAS"</SectionTitle>
                </div>

                <Suspense fallback=move || view! {
                    <Card>
                        <div class="flex-col flex-center gap-6" style="padding: var(--sp-12) 0">
                            <span class="spinner" style="width: 40px; height: 40px; border-width: 3px" />
                            <p class="t-mono-xs t-muted">"Carregando suas cooperativas…"</p>
                        </div>
                    </Card>
                }>
                    {move || coops.get().map(|res| match res {
                        Err(e) => view! {
                            <Alert kind=AlertKind::Error>{e.to_string()}</Alert>
                        }.into_any(),

                        Ok(list) if list.is_empty() => view! {
                            <Card>
                                <span class="card-tag">"NENHUMA COOPERATIVA"</span>
                                <div class="flex-col gap-4" style="margin-top: var(--sp-6)">
                                    <p class="t-mono-xs t-muted">
                                        "Você ainda não faz parte de nenhuma cooperativa. "
                                        "Vincule sua carteira a uma cooperativa existente "
                                        "ou crie a sua."
                                    </p>
                                    <div style="display: flex; gap: var(--sp-3); flex-wrap: wrap">
                                        <LinkTag href="/vinculation".to_string() color=LinkTagColor::Yellow>
                                            "Vincular-se"
                                        </LinkTag>
                                        <LinkTag href="/create-coop".to_string() color=LinkTagColor::Gold>
                                            "Criar cooperativa"
                                        </LinkTag>
                                    </div>
                                </div>
                            </Card>
                        }.into_any(),

                        Ok(list) => view! {
                            <div class="flex-col gap-6">
                                {list.into_iter().map(|c| view! {
                                    <CoopRow coop=c />
                                }).collect_view()}
                            </div>
                        }.into_any(),
                    })}
                </Suspense>
            </div>
        </Section>
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CoopRow
// ─────────────────────────────────────────────────────────────────────────────
//
// Per-coop card.  All action links route on `coop_id` (the bytes32 hex from
// the registry) — that's the public identifier your other pages already use
// (e.g. ElectionsPage takes coop_id).

#[component]
fn CoopRow(coop: UserCoop) -> impl IntoView {
    let coop_id = coop.coop_id.clone();
    let elections_href = format!("/elections/{}", coop_id);
    let panel_href     = format!("/cooperatives/{}", coop_id);

    view! {
        <Card>
            <div class="flex-col gap-6">

                // ── Header: name + status ──────────────────────────
                <div style="display: flex; align-items: center; justify-content: space-between; gap: var(--sp-4); flex-wrap: wrap">
                    <span class="card-tag">{coop.name.clone()}</span>
                    {if coop.active {
                        view! {
                            <Badge color=BadgeColor::Green filled=true>"ATIVA"</Badge>
                        }.into_any()
                    } else {
                        view! {
                            <Badge color=BadgeColor::Red>"INATIVA"</Badge>
                        }.into_any()
                    }}
                </div>

                // ── Identifiers ────────────────────────────────────
                <div class="form-group">
                    <label class="form-label">"ID da Cooperativa"</label>
                    <HashDisplay value=coop.coop_id.clone() />
                </div>

                <div class="form-group">
                    <label class="form-label">"Endereço do Contrato"</label>
                    <HashDisplay value=coop.loan_machine />
                </div>

                // ── Actions per coop ───────────────────────────────
                // For now: control panel + elections.
                // Add donations / loans / withdrawals here as those land.
                <div class="form-group">
                    <label class="form-label">"Ações"</label>
                    <div style="display: flex; gap: var(--sp-3); flex-wrap: wrap; margin-top: var(--sp-3)">
                        <LinkTag href=panel_href     color=LinkTagColor::Gold   size=LinkTagSize::Sm>
                            "Painel"
                        </LinkTag>
                        <LinkTag href=elections_href color=LinkTagColor::Yellow size=LinkTagSize::Sm>
                            "Eleições"
                        </LinkTag>
                        // TODO: Donations, Loans, Withdrawals…
                    </div>
                </div>

            </div>
        </Card>
    }
}