// loan_machine_core/src/server_logic/subgraph_queries/last_closed_election.rs

use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

use loan_machine_models::responses::ElectionView;
use crate::services::subgraph::{SubgraphError, SubgraphService};
use crate::server_logic::helpers::resolve_member_wallet;

const QUERY: &str = r#"
query LastClosedElection($coopId: String!) {
  electionClosedEvents(
    where: { cooperative: $coopId }
    orderBy: blockTimestamp
    orderDirection: desc
    first: 1
  ) {
    electionId
    winnerId
    winningVotes
    blockTimestamp
  }
}"#;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Vars { coop_id: String }

#[derive(Deserialize)]
struct Row {
    #[serde(rename = "electionId")]     election_id:    i32,
    #[serde(rename = "winnerId")]       winner_id:      String,
    #[serde(rename = "winningVotes")]   winning_votes:  i32,
    #[serde(rename = "blockTimestamp")] block_timestamp: String,
}

#[derive(Deserialize)]
struct Data {
    #[serde(rename = "electionClosedEvents")]
    events: Vec<Row>,
}

/// None when the coop has never closed an election.
/// `winner_id` is translated to a wallet hex via the subgraph; falls back to
/// the raw memberId hex if translation fails (e.g. indexer race).
pub async fn fetch_last_closed_election(
    subgraph:          &SubgraphService,
    loan_machine_addr: Address,
) -> Result<Option<ElectionView>, SubgraphError> {
    let coop_id = format!("{loan_machine_addr:#x}").to_lowercase();
    let data: Data = subgraph.query(QUERY, Vars { coop_id }).await?;
    let Some(row)  = data.events.into_iter().next() else { return Ok(None); };

    let winner_b32 = row.winner_id.parse().ok();
    let winner_pretty = match winner_b32 {
        Some(b) => resolve_member_wallet(subgraph, loan_machine_addr, b)
            .await
            .map(|w| w.to_string())
            .unwrap_or(row.winner_id),
        None => row.winner_id,
    };

    let end_time = row.block_timestamp.parse::<u64>().unwrap_or(0);

    Ok(Some(ElectionView {
        id:               row.election_id as u32,
        candidates:       Vec::new(),     // not stored on the close event
        candidate_votes:  Vec::new(),     // ditto
        start_time:       0,              // ditto — only have close time
        end_time,
        is_active:        false,
        winner_id:        winner_pretty,
        winning_votes:    row.winning_votes,
        total_votes_cast: 0,              // not on this event
    }))
}

