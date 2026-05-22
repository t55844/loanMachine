// loan_machine_core/src/server_logic/helpers/resolve_member_reputation.rs

use alloy::primitives::{Address, B256};
use serde::{Deserialize, Serialize};

use crate::services::blockchain::{abis::LoanMachine, BlockchainError};
use crate::services::subgraph::SubgraphService;
use crate::services::blockchain::Provider;

const QUERY: &str = r#"
query MemberReputation($coopId: String!, $memberId: Bytes!) {
  ReputationChangedEvents(
    where: { cooperative: $coopId, memberId: $memberId }
    orderBy: blockTimestamp
    orderDirection: desc
    first: 1
  ) {
    newReputation
  }
}"#;

#[derive(Serialize)]
struct Vars<'a> {
    #[serde(rename = "coopId")]   coop_id:   &'a str,
    #[serde(rename = "memberId")] member_id: &'a str,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Row {
    new_reputation: i32,
}

#[derive(Deserialize)]
struct Data {
    #[serde(rename = "ReputationChangedEvents")]
    events: Vec<Row>,
}

async fn subgraph_lookup(
    subgraph:      &SubgraphService,
    coop_id:       &str,
    member_id_hex: &str,
) -> Option<i32> {
    let vars = Vars { coop_id, member_id: member_id_hex };
    let data: Data = subgraph.query(QUERY, vars).await.ok()?;
    data.events.into_iter().next().map(|r| r.new_reputation)
}


async fn on_chain_scan(
    provider:          &Provider,
    loan_machine_addr: Address,
    member_id:         B256,
) -> Result<i32, BlockchainError> {
    let contract = LoanMachine::new(loan_machine_addr, provider.clone());
    let rep = contract
        .getReputation(member_id)
        .call()
        .await
        .map_err(BlockchainError::from_call)?
        ._0;
    Ok(rep)
}

pub async fn resolve_member_reputation(
    subgraph:          &SubgraphService,
    provider:          &Provider,
    loan_machine_addr: Address,
    member_id:         B256,
) -> Result<i32, BlockchainError> {
    let coop_id    = format!("{loan_machine_addr:#x}").to_lowercase();
    let member_hex = format!("0x{}", hex::encode(member_id.as_slice()));

    if let Some(rep) = subgraph_lookup(subgraph, &coop_id, &member_hex).await {
        return Ok(rep);
    }
    on_chain_scan(provider, loan_machine_addr, member_id).await
}