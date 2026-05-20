// loan_machine_web/src/components/elections/current_election.rs
//
// Owns the full open-election domain:
//   • form signals (candidate, opponent, validation errors)
//   • the prepare_open_election Action
//   • Effect: bundle arrives → gas modal
//   • Effect: server error → error signal
//   • on_tx_outcome subscription → success bubbles up via on_tx_success
//
// The parent (ElectionsPage) only receives on_tx_success(()) so it can
// re-run the current-election Resource.  Everything else is local.
//
// Branches on `current: Option<ElectionView>`:
//   None → StartElectionStep — the form
//   Some → ActiveElectionStep — read-only summary (vote/close: TODO)

use leptos::prelude::*;

use loan_machine_models::responses::ElectionView;
use loan_machine_models::wallet_address::WalletAddress;

use crate::components::ui::*;
use crate::components::gas_modal::{use_gas_modal, GasEstimate, GasModalRequest};
use crate::server_fns::elections::prepare_open_election;
use crate::wallet_auth::privy_bridge::{self, TxOutcome};

use crate::components::elections::vote_election::VoteElection;

#[component]
pub fn CurrentElection(
    #[prop(into)] coop_id:       String,
                  current:       Option<ElectionView>,
                  reputation:    Option<i32>,   // None = loading / unvinculated
    on_tx_success: Callback<()>,
) -> impl IntoView {
    match current {
        Some(v) => ActiveElectionStep(ActiveElectionStepProps {
            coop_id, view_data: v, reputation, on_tx_success,
        }).into_any(),
        None => StartElectionStep(StartElectionStepProps {
            coop_id, on_tx_success,
        }).into_any(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// StartElectionStep
// ─────────────────────────────────────────────────────────────────────────────

#[component]
fn StartElectionStep(
    #[prop(into)] coop_id: String,
    on_tx_success: Callback<()>,
) -> impl IntoView {
    let (candidate, set_candidate) = signal(String::new());
    let (opponent,  set_opponent)  = signal(String::new());

    let (candidate_err, set_candidate_err) = signal(String::new());
    let (opponent_err,  set_opponent_err ) = signal(String::new());
    let (error,         set_error        ) = signal(String::new());

    let open_election = Action::new(
        move |(coop_id, cand, opp): &(String, WalletAddress, WalletAddress)| {
            let coop_id = coop_id.clone();
            let cand    = *cand;
            let opp     = *opp;
            async move { prepare_open_election(coop_id, cand, opp).await }
        }
    );
    let loading = open_election.pending();

    // Bundle arrives → gas modal
    let gas_modal = use_gas_modal();
    Effect::new(move |_| {
        let Some(Ok(b)) = open_election.value().get() else { return };
        let to   = b.to.clone();
        let data = b.data.clone();
        let gas  = b.gas_hex.clone();
        gas_modal.set(Some(GasModalRequest {
            title: "CONFIRMAR ABERTURA DE ELEIÇÃO".into(),
            estimates: vec![GasEstimate {
                label:   "Abrir eleição de moderador".into(),
                gas_hex: gas.clone(),
            }],
            on_confirm: Callback::new(move |_| {
                privy_bridge::send_tx(&to, &data, Some(&gas));
            }),
        }));
    });

    // Server error
    Effect::new(move |_| {
        if let Some(Err(e)) = open_election.value().get() {
            set_error.set(e.to_string());
        }
    });

    // on-chain outcome
    privy_bridge::on_tx_outcome(move |outcome| match outcome {
        TxOutcome::Complete(_) => {
            set_error.set(String::new());
            on_tx_success.run(());
        }
        TxOutcome::Failed(err) => {
            set_error.set(format!("Falha na transação: {err}"));
        }
    });

    view! {
        <Card>
            <span class="card-tag">"NENHUMA ELEIÇÃO ATIVA"</span>
            <div class="flex-col gap-6" style="margin-top: var(--sp-6)">

                <p class="t-mono-xs t-muted">
                    "Inicie uma eleição fornecendo os endereços de carteira dos dois
                    candidatos. Ambos devem ser membros vinculados desta cooperativa."
                </p>

                {move || {
                    let e = error.get();
                    (!e.is_empty()).then(|| view! {
                        <Alert kind=AlertKind::Error>{e}</Alert>
                    })
                }}

                <TextInput
                    label="Candidato — Carteira"
                    placeholder="0x0000...0000"
                    hint="Membro vinculado desta cooperativa"
                    value=candidate
                    set_value=set_candidate
                    error=Signal::derive(move || candidate_err.get())
                />

                <TextInput
                    label="Oponente — Carteira"
                    placeholder="0x0000...0000"
                    hint="Segundo candidato, também membro vinculado"
                    value=opponent
                    set_value=set_opponent
                    error=Signal::derive(move || opponent_err.get())
                />

                <Button
                    variant=BtnVariant::Primary
                    full_width=true
                    loading=Signal::derive(move || loading.get())
                    on_click=Box::new(move || {
                        let cand_raw = candidate.get_untracked();
                        let opp_raw  = opponent.get_untracked();

                        let cand: Result<WalletAddress, _> = cand_raw.parse();
                        let opp:  Result<WalletAddress, _> = opp_raw.parse();

                        set_candidate_err.set(match &cand {
                            Ok(_)  => String::new(),
                            Err(_) => "Endereço inválido (0x + 40 hex)".into(),
                        });
                        set_opponent_err.set(match &opp {
                            Ok(_)  => String::new(),
                            Err(_) => "Endereço inválido (0x + 40 hex)".into(),
                        });

                        if let (Ok(c), Ok(o)) = (cand, opp) {
                            if c == o {
                                set_opponent_err.set(
                                    "Oponente deve ser diferente do candidato".into()
                                );
                                return;
                            }
                            set_error.set(String::new());
                            open_election.dispatch((coop_id.clone(), c, o));
                        }
                    })
                >
                    "INICIAR ELEIÇÃO"
                </Button>
            </div>
        </Card>
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ActiveElectionStep
// ─────────────────────────────────────────────────────────────────────────────


#[component]
fn ActiveElectionStep(
    #[prop(into)] coop_id:       String,
                  view_data:     ElectionView,
                  reputation:    Option<i32>,
    on_tx_success: Callback<()>,
) -> impl IntoView {
    let candidate_count     = view_data.candidates.len();

    let rep_label = match reputation {
        Some(r) => r.to_string(),
        None    => "—".into(),
    };
    let zero_rep_warning = matches!(reputation, Some(0));

    view! {
        <Card variant=CardVariant::Gold>
            <span class="card-tag">{format!("ELEIÇÃO #{} — EM ANDAMENTO", view_data.id)}</span>
            <div class="flex-col gap-6" style="margin-top: var(--sp-6)">

                <Alert kind=AlertKind::Info>
                    "A eleição está aberta. O peso do seu voto é igual à sua reputação."
                </Alert>

                {zero_rep_warning.then(|| view! {
                    <Alert kind=AlertKind::Warning>
                        "Você tem reputação zero — seu voto será registrado mas não
                         influenciará o resultado. Cubra um empréstimo ou quite uma
                         parcela em dia para ganhar reputação."
                    </Alert>
                })}

                <div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: var(--sp-4)">
                    <StatBlock label="Sua reputação"  value=rep_label />
                    <StatBlock label="Total de votos" value=view_data.total_votes_cast.to_string() />
                    <StatBlock label="Encerramento"   value=format_epoch(view_data.end_time) />
                </div>

                <div class="form-group">
                    <label class="form-label">
                        {format!("Candidatos ({})", candidate_count)}
                    </label>
                    <div class="flex-col gap-3" style="margin-top: var(--sp-3)">
                        {view_data.candidates.into_iter().enumerate().map(|(i, c)| view! {
                            <div style="display: flex; align-items: center; gap: var(--sp-3)">
                                <span
                                    class="badge badge-yellow"
                                    style="flex-shrink: 0; min-width: 1.8rem; text-align: center"
                                >
                                    {(i + 1).to_string()}
                                </span>
                                <HashDisplay value=c />
                            </div>
                        }).collect_view()}
                    </div>
                </div>

                <VoteElection
                    coop_id=coop_id
                    election_id=view_data.id
                    on_tx_success=on_tx_success
                />
            </div>
        </Card>
    }
}
// ─────────────────────────────────────────────────────────────────────────────
// helpers
// ─────────────────────────────────────────────────────────────────────────────

fn format_epoch(ts: u64) -> String {
    let days              = ts / 86_400;
    let epoch_day_offset  = 719_162u64;
    let day               = days + epoch_day_offset;
    let year              = day / 365;
    let month             = (day % 365) / 30 + 1;
    let dom               = (day % 365) % 30 + 1;
    format!("{year}-{month:02}-{dom:02}")
}