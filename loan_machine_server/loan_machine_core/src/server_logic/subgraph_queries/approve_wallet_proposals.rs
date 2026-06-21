use std::collections::HashSet;
use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

use crate::services::subgraph::{SubgraphError, SubgraphService};

const QUERY: &str = r#"
query ApproveWalletProposals($coopId: String!) {
  created: proposalCreatedEvents(
    where: { cooperative: $coopId, pType: 2 }
    orderBy: blockTimestamp, orderDirection: asc
  ) { proposalId proposer blockTimestamp }
  executed: proposalExecutedEvents(where: { cooperative: $coopId, pType: 2 }) {
    proposalId
  }
  confirmed: proposalConfirmedEvents(where: { cooperative: $coopId }) {
    proposalId admin
  }
}"#;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Vars { coop_id: String }

#[derive(Deserialize)]
struct CreatedRow {
    #[serde(rename = "proposalId")]     proposal_id: String,
    proposer:                           String,
    #[serde(rename = "blockTimestamp")] block_timestamp: String,
}
#[derive(Deserialize)] struct IdRow { #[serde(rename = "proposalId")] proposal_id: String }
#[derive(Deserialize)]
struct ConfRow {
    #[serde(rename = "proposalId")] proposal_id: String,
    admin: String,
}
#[derive(Deserialize)]
struct Data {
    created:   Vec<CreatedRow>,
    executed:  Vec<IdRow>,
    confirmed: Vec<ConfRow>,
}

pub struct ApproveWalletProposalRaw {
    pub proposal_id:      u64,
    pub proposer:         String,
    pub created_at:       u64,
    pub confirmations:    u32,
    pub viewer_confirmed: bool,
}

pub async fn fetch_pending_approve_wallet_proposals(
    subgraph:          &SubgraphService,
    loan_machine_addr: Address,
    viewer_wallet:     Address,
) -> Result<Vec<ApproveWalletProposalRaw>, SubgraphError> {
    let coop_id    = format!("{loan_machine_addr:#x}").to_lowercase();
    let viewer_hex = format!("{viewer_wallet:#x}").to_lowercase();
    let data: Data = subgraph.query(QUERY, Vars { coop_id }).await?;

    let executed: HashSet<&str> = data.executed.iter().map(|r| r.proposal_id.as_str()).collect();

    let mut out = Vec::new();
    for c in &data.created {
        let id = c.proposal_id.as_str();
        if executed.contains(id) { continue; }

        let confirmations = data.confirmed.iter()
            .filter(|r| r.proposal_id == c.proposal_id).count() as u32;

        let viewer_confirmed = data.confirmed.iter().any(|r|
            r.proposal_id == c.proposal_id && r.admin.to_lowercase() == viewer_hex
        );

        out.push(ApproveWalletProposalRaw {
            proposal_id:  c.proposal_id.parse().unwrap_or(0),
            proposer:     c.proposer.clone(),
            created_at:   c.block_timestamp.parse().unwrap_or(0),
            confirmations,
            viewer_confirmed,
        });
    }
    Ok(out)
}