// loan_machine_web/src/components/elections/last_result.rs


use leptos::prelude::*;

use loan_machine_models::responses::ElectionView;

use crate::components::ui::*;

#[component]
pub fn LastResult(
    /// None → show "no results yet" placeholder. Some → show closed-election data.
    last: Option<ElectionView>,
) -> impl IntoView {
    match last {
        None    => EmptyResultStep().into_any(),
        Some(v) => ClosedElectionStep(ClosedElectionStepProps { view_data: v }).into_any(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// EmptyResultStep
// ─────────────────────────────────────────────────────────────────────────────

#[component]
fn EmptyResultStep() -> impl IntoView {
    view! {
        <Card>
            <span class="card-tag">"ÚLTIMO RESULTADO"</span>
            <div class="flex-col gap-4" style="margin-top: var(--sp-6)">
                <p class="t-mono-xs t-muted">
                    "Nenhuma eleição encerrada ainda. O resultado da primeira eleição
                    aparecerá aqui assim que for concluída."
                </p>
            </div>
        </Card>
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ClosedElectionStep
// ─────────────────────────────────────────────────────────────────────────────

#[component]
fn ClosedElectionStep(view_data: ElectionView) -> impl IntoView {
    // winner_id is bytes32 hex; a zero hash means the election closed
    // without any votes cast (edge case — show "N/A" rather than a hash).
    let winner_is_zero = view_data.winner_id == "0x0000000000000000000000000000000000000000000000000000000000000000";

    view! {
        <Card variant=CardVariant::Yellow>
            <span class="card-tag">{format!("ÚLTIMO RESULTADO — ELEIÇÃO #{}", view_data.id)}</span>
            <div class="flex-col gap-6" style="margin-top: var(--sp-6)">

                // ── Winner ───────────────────────────────────────────
                <div class="form-group">
                    <label class="form-label">"Vencedor (Carteira)"</label>
                    <div style="margin-top: var(--sp-3)">
                        {if winner_is_zero {
                            view! { <span class="t-mono-xs t-muted">"N/A — sem votos"</span> }.into_any()
                        } else {
                            view! { <HashDisplay value=view_data.winner_id /> }.into_any()
                        }}
                    </div>
                    <span class="form-hint">
                        "Este membro agora é moderador da cooperativa."
                    </span>
                </div>

                // ── Vote counts ──────────────────────────────────────
                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: var(--sp-4)">
                    <StatBlock
                        label="Votos vencedores"
                        value=view_data.winning_votes.to_string()
                    />
                    <StatBlock
                        label="Total computado"
                        value=view_data.total_votes_cast.to_string()
                    />
                </div>

                // ── Status badge ─────────────────────────────────────
                <div style="display: flex; align-items: center; gap: var(--sp-3)">
                    <span class="stat-label">"Status"</span>
                    <Badge color=BadgeColor::Green filled=true>
                        "ENCERRADA"
                    </Badge>
                </div>

            </div>
        </Card>
    }
}