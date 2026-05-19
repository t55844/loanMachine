// loan_machine_core/src/server_logic/subgraph_queries/cooperative_by_id.rs

use serde::Serialize;

use crate::services::subgraph::{SubgraphError, SubgraphService};
use crate::server_logic::subgraph_queries::cooperatives::CooperativeRow;

const QUERY: &str = r#"
query CooperativeById($coopId: Bytes!) {
  cooperatives(where: { coopId: $coopId }, first: 1) {
    id
    coopId
    name
    loanMachine
    active
    registeredAt
  }
}"#;

#[derive(Serialize)]
struct Vars<'a> {
    #[serde(rename = "coopId")]
    coop_id: &'a str,
}

#[derive(serde::Deserialize)]
struct Data {
    cooperatives: Vec<CooperativeRow>,
}

/// Returns the coop if found, None if no coop has this id in the index.
pub async fn fetch_one_coop(
    subgraph: &SubgraphService,
    coop_id: &str,
) -> Result<Option<CooperativeRow>, SubgraphError> {
    let data: Data = subgraph.query(QUERY, Vars { coop_id }).await?;
    Ok(data.cooperatives.into_iter().next())
}