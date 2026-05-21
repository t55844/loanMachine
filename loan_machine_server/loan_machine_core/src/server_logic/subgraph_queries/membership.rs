// loan_machine_core/src/server_logic/subgraph_queries/membership.rs
//
// Is `wallet` registered as a member of `coop_id`?
//
// Strategy: filter MemberRegisteredEvent by cooperative.coopId + wallet.
// One event exists per (member, wallet) registration, so any hit proves
// membership. Returns false if the indexer is behind — the on-chain
// state is still the source of truth.

use serde::{Deserialize, Serialize};

use loan_machine_models::wallet_address::WalletAddress;
use crate::services::subgraph::{SubgraphError, SubgraphService};

const QUERY: &str = r#"
query IsMember($coopId: Bytes!, $wallet: Bytes!) {
  memberRegisteredEvents(
    where: {
      cooperative_: { coopId: $coopId },
      wallet: $wallet
    }
    first: 1
  ) {
    id
  }
}"#;

#[derive(Serialize)]
struct Vars<'a> {
    #[serde(rename = "coopId")] coop_id: &'a str,
    wallet: &'a str,
}

#[derive(Deserialize)]
struct Row { #[allow(dead_code)] id: String }

#[derive(Deserialize)]
struct Data {
    #[serde(rename = "memberRegisteredEvents")]
    events: Vec<Row>,
}

pub async fn is_member(
    subgraph: &SubgraphService,
    coop_id:  &str,
    wallet:   &WalletAddress,
) -> Result<bool, SubgraphError> {
    let wallet_hex = wallet.to_string().to_lowercase();   // see note below
    let data: Data = subgraph
        .query(QUERY, Vars { coop_id, wallet: &wallet_hex })
        .await?;
    Ok(!data.events.is_empty())
}