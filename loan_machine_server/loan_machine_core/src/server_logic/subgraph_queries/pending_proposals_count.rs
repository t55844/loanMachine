use alloy::primitives::Address;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::services::subgraph::{SubgraphError, SubgraphService};

const QUERY: &str = r#"
query PendingForUser($coopId: String!, $wallet: Bytes!) {
  created: proposalCreatedEvents(where: {cooperative: $coopId, pType: 2}) {
    proposalId
  }
  executed: proposalExecutedEvents(where: {cooperative: $coopId, pType: 2}) {
    proposalId
  }
  confirmedByMe: proposalConfirmedEvents(where: {cooperative: $coopId, admin: $wallet}) {
    proposalId
  }
  cosigned: proposalCosignedEvents(where: {cooperative: $coopId}) {
    proposalId
  }
}"#;

#[derive(Serialize)]
struct Vars<'a> {
    #[serde(rename = "coopId")] coop_id: String,
                                wallet:  &'a str,
}

#[derive(Deserialize)]
struct Row { #[serde(rename = "proposalId")] proposal_id: String }

#[derive(Deserialize)]
struct Data {
    created:                                              Vec<Row>,
    executed:                                             Vec<Row>,
    #[serde(rename = "confirmedByMe")] confirmed_by_me:   Vec<Row>,
    cosigned:                                             Vec<Row>,
}

/// Count ApproveWallet proposals where the viewer has an action to take.
/// Skips the round-trip entirely when the viewer has no role.
pub async fn count_actionable_proposals(
    subgraph:          &SubgraphService,
    loan_machine_addr: Address,
    wallet:            Address,
    is_admin:          bool,
    is_moderator:      bool,
) -> Result<u32, SubgraphError> {
    if !is_admin && !is_moderator { return Ok(0); }

    let coop_id    = format!("{loan_machine_addr:#x}").to_lowercase();
    let wallet_hex = format!("{wallet:#x}").to_lowercase();
    let vars       = Vars { coop_id, wallet: &wallet_hex };

    let data: Data = subgraph.query(QUERY, vars).await?;

    let executed:  HashSet<&str> = data.executed.iter().map(|r| r.proposal_id.as_str()).collect();
    let confirmed: HashSet<&str> = data.confirmed_by_me.iter().map(|r| r.proposal_id.as_str()).collect();
    let cosigned:  HashSet<&str> = data.cosigned.iter().map(|r| r.proposal_id.as_str()).collect();

    let mut count = 0u32;
    for c in &data.created {
        let id = c.proposal_id.as_str();
        if executed.contains(id) { continue; }
        let admin_action = is_admin     && !confirmed.contains(id);
        let mod_action   = is_moderator && !cosigned.contains(id);
        if admin_action || mod_action { count += 1; }
    }
    Ok(count)
}