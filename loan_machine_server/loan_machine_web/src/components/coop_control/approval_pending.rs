// loan_machine_web/src/components/coop_approval/approval_pending.rs

use leptos::prelude::*;

use loan_machine_models::responses::ApprovalStatus;
use crate::components::ui::*;
use crate::server_fns::coop_approval::get_approval_status;

#[component]
pub fn ApprovalPending(
    #[prop(into)] coop_id: String,
) -> impl IntoView {
    let coop_id_for_res = coop_id.clone();
    let status_res = Resource::new(
        move || coop_id_for_res.clone(),
        |id| async move { get_approval_status(id).await.ok() },
    );

    view! {
        <Card tag="PENDING APPROVAL" hover=false>
            <Suspense fallback=move || view! {
                <div class="flex-center" style="padding: var(--sp-8) 0">
                    <span class="spinner"></span>
                </div>
            }>
                {move || status_res.get().flatten().map(|status| match status {
                    ApprovalStatus::Pending {
                        proposal_id, confirmations, threshold,
                        total_admins, moderator_cosigned, ..
                    } => view! {
                        <div class="flex-col gap-4" style="margin-top: var(--sp-4)">
                            <p class="t-mono-xs t-muted">
                                "Your request has been registered. Wait until the administrators
                                confirm and a moderator co-signs."
                            </p>

                            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: var(--sp-4)">
                                <StatBlock
                                    label="Admin confirmations"
                                    value=format!("{}/{} (threshold {})", confirmations, total_admins, threshold)
                                />
                                <StatBlock
                                    label="Moderator co-signed"
                                    value=if moderator_cosigned { "Yes".into() } else { "No".into() }
                                />
                            </div>

                            <div class="stat-block">
                                <span class="stat-label">"Proposal ID"</span>
                                <span class="t-mono-sm">{proposal_id.to_string()}</span>
                            </div>
                        </div>
                    }.into_any(),

                    _ => view! {
                        <p class="t-mono-xs t-muted" style="margin-top: var(--sp-4)">
                            "State updating..."
                        </p>
                    }.into_any(),
                })}
            </Suspense>
        </Card>
    }
}