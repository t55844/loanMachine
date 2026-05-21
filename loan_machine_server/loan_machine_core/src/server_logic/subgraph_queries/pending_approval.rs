// loan_machine_core/src/server_logic/subgraph_queries/pending_approval.rs

use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

use crate::services::subgraph::{SubgraphError, SubgraphService};

const QUERY: &str = r#"
query PendingApproval($coopId: String!, $wallet: Bytes!) {
  proposalCreatedEvents(
    where: {
      cooperative: $coopId,
      pType: 2,
      proposer: $wallet
    }
    orderBy: blockTimestamp
    orderDirection: desc
    first: 1
  ) {
    proposalId
    blockTimestamp
  }
}"#;

#[derive(Serialize)]
struct Vars<'a> {
    #[serde(rename = "coopId")] coop_id: String,
                                wallet:  &'a str,
}

#[derive(Deserialize)]
struct Row {
    #[serde(rename = "proposalId")]     proposal_id:    String,
    #[serde(rename = "blockTimestamp")] block_timestamp: String,
}

#[derive(Deserialize)]
struct Data {
    #[serde(rename = "proposalCreatedEvents")]
    events: Vec<Row>,
}

pub struct PendingApproval {
    pub proposal_id:     u64,
    pub created_at:      u64,
}

/// Returns the most recent ApproveWallet proposal for this wallet in this coop,
/// or None if none exists. Caller checks executed/cosigned state from chain.
pub async fn fetch_pending_approval(
    subgraph:          &SubgraphService,
    loan_machine_addr: Address,
    wallet:            Address,
) -> Result<Option<PendingApproval>, SubgraphError> {
    let coop_id    = format!("{loan_machine_addr:#x}").to_lowercase();
    let wallet_hex = format!("{wallet:#x}").to_lowercase();
    let vars       = Vars { coop_id, wallet: &wallet_hex };

    let data: Data = subgraph.query(QUERY, vars).await?;
    let Some(row)  = data.events.into_iter().next() else { return Ok(None); };

    Ok(Some(PendingApproval {
        proposal_id: row.proposal_id.parse().unwrap_or(0),
        created_at:  row.block_timestamp.parse().unwrap_or(0),
    }))
}