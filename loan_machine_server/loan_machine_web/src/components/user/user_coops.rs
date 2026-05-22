use leptos::prelude::*;

use loan_machine_models::responses::{UserProfileCoop};

use crate::components::ui::*;
use crate::server_fns::user_profile::get_user_profile;

#[component]
pub fn UserCoopsPage() -> impl IntoView {
    let profile = Resource::new(|| (), |_| async move { get_user_profile().await });

    view! {
        <Section>
            <div class="container-md">
                <div class="t-center" style="margin-bottom: var(--sp-8)">
                    <SectionTitle>"MEU PERFIL"</SectionTitle>
                </div>

                <Suspense fallback=move || view! {
                    <Card>
                        <div class="flex-col flex-center gap-6" style="padding: var(--sp-12) 0">
                            <span class="spinner" style="width: 40px; height: 40px; border-width: 3px" />
                            <p class="t-mono-xs t-muted">"Carregando seu perfil…"</p>
                        </div>
                    </Card>
                }>
                    {move || profile.get().map(|res| match res {
                        Err(e) => view! { <Alert kind=AlertKind::Error>{e.to_string()}</Alert> }.into_any(),
                        Ok(p) if p.coops.is_empty() => view! { <EmptyState /> }.into_any(),
                        Ok(p) => view! {
                            <ProfileHeader wallet=p.wallet.clone() coops=p.coops.clone() />
                            <div class="flex-col gap-6" style="margin-top: var(--sp-6)">
                                {p.coops.into_iter().map(|c| view! { <CoopRow coop=c /> }).collect_view()}
                            </div>
                        }.into_any(),
                    })}
                </Suspense>
            </div>
        </Section>
    }
}

#[component]
fn ProfileHeader(wallet: String, coops: Vec<UserProfileCoop>) -> impl IntoView {
    let total_pending: u32 = coops.iter().map(|c| c.pending_count).sum();
    let admin_count = coops.iter().filter(|c| c.is_admin).count();
    let mod_count   = coops.iter().filter(|c| c.is_moderator).count();

    view! {
        <Card>
            <div class="flex-col gap-4">
                <span class="card-tag">"CARTEIRA"</span>
                <HashDisplay value=wallet />
                <div style="display: flex; gap: var(--sp-3); flex-wrap: wrap; margin-top: var(--sp-2)">
                    {(admin_count > 0).then(|| view! {
                        <Badge color=BadgeColor::Gold filled=true>
                            {format!("ADMIN EM {admin_count}")}
                        </Badge>
                    })}
                    {(mod_count > 0).then(|| view! {
                        <Badge color=BadgeColor::Yellow filled=true>
                            {format!("MODERADOR EM {mod_count}")}
                        </Badge>
                    })}
                    {(total_pending > 0).then(|| view! {
                        <Badge color=BadgeColor::Red filled=true live=true>
                            {format!("{total_pending} PENDENTE{}", if total_pending == 1 { "" } else { "S" })}
                        </Badge>
                    })}
                </div>
            </div>
        </Card>
    }
}

#[component]
fn CoopRow(coop: UserProfileCoop) -> impl IntoView {
    let panel_href     = format!("/cooperatives/{}", coop.coop_id);
    let elections_href = format!("/elections/{}",    coop.coop_id);
    let is_member      = coop.is_member;
    let has_role       = coop.is_admin || coop.is_moderator;
    let has_pending    = coop.pending_count > 0;

    // Relation badge — only one of these renders, in priority order.
    let relation_badge = if is_member {
        view! { <Badge color=BadgeColor::Green filled=true>"MEMBRO"</Badge> }.into_any()
    } else if coop.is_approved {
        view! { <Badge color=BadgeColor::Yellow filled=true>"APROVADO — VINCULAR"</Badge> }.into_any()
    } else if coop.has_pending_approval {
        view! { <Badge color=BadgeColor::Gold>"AGUARDANDO APROVAÇÃO"</Badge> }.into_any()
    } else {
        view! { <Badge color=BadgeColor::Red>"SEM RELAÇÃO"</Badge> }.into_any()
    };

    view! {
        <Card>
            <div class="flex-col gap-6">

                <div style="display: flex; align-items: center; justify-content: space-between; gap: var(--sp-4); flex-wrap: wrap">
                    <span class="card-tag">{coop.name.clone()}</span>
                    <div style="display: flex; gap: var(--sp-2); flex-wrap: wrap">
                        {relation_badge}
                        {if coop.active {
                            view! { <Badge color=BadgeColor::Green filled=true>"ATIVA"</Badge> }.into_any()
                        } else {
                            view! { <Badge color=BadgeColor::Red>"INATIVA"</Badge> }.into_any()
                        }}
                        {has_pending.then(|| view! {
                            <Badge color=BadgeColor::Red filled=true live=true>
                                {format!("{} PENDENTE{}", coop.pending_count, if coop.pending_count == 1 { "" } else { "S" })}
                            </Badge>
                        })}
                    </div>
                </div>

                {has_role.then(|| view! {
                    <div style="display: flex; gap: var(--sp-2); flex-wrap: wrap">
                        {coop.is_admin.then(|| view! {
                            <Badge color=BadgeColor::Gold filled=true>"ADMIN"</Badge>
                        })}
                        {coop.is_moderator.then(|| view! {
                            <Badge color=BadgeColor::Yellow filled=true>"MODERADOR"</Badge>
                        })}
                    </div>
                })}

                
                <div class="form-group">
                    <label class="form-label">"Meu Member ID"</label>
                    <HashDisplay value=coop.member_id.clone() />
                </div>
                

                <div class="form-group">
                    <label class="form-label">"ID da Cooperativa"</label>
                    <HashDisplay value=coop.coop_id.clone() />
                </div>

                <div class="form-group">
                    <label class="form-label">"Ações"</label>
                    <div style="display: flex; gap: var(--sp-3); flex-wrap: wrap; margin-top: var(--sp-3)">
                        <LinkTag href=panel_href color=LinkTagColor::Gold size=LinkTagSize::Sm>
                            {if has_pending { "Painel · Ações pendentes" }
                             else if !is_member && coop.is_approved { "Vincular agora" }
                             else if !is_member && coop.has_pending_approval { "Ver status" }
                             else { "Painel" }}
                        </LinkTag>
                        {is_member.then(|| view! {
                            <LinkTag href=elections_href color=LinkTagColor::Yellow size=LinkTagSize::Sm>
                                "Eleições"
                            </LinkTag>
                        })}
                    </div>
                </div>
            </div>
        </Card>
    }
}

#[component]
fn EmptyState() -> impl IntoView {
    view! {
        <Card>
            <span class="card-tag">"NENHUMA COOPERATIVA"</span>
            <div class="flex-col gap-4" style="margin-top: var(--sp-6)">
                <p class="t-mono-xs t-muted">
                    "Você ainda não faz parte de nenhuma cooperativa. "
                    "Vincule sua carteira a uma cooperativa existente ou crie a sua."
                </p>
                <div style="display: flex; gap: var(--sp-3); flex-wrap: wrap">
                    <LinkTag href="/vinculation".to_string() color=LinkTagColor::Yellow>"Vincular-se"</LinkTag>
                    <LinkTag href="/create-coop".to_string() color=LinkTagColor::Gold>"Criar cooperativa"</LinkTag>
                </div>
            </div>
        </Card>
    }
}