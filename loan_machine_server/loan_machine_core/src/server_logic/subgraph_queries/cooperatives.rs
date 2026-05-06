/// loan_machine_server/loan_machine_core/src/server_logic/subgraph_queries/cooperatives.rs
/// logic to get all coperatives

use serde::{Serialize,Deserialize};

use crate::services::subgraph::{SubgraphError, SubgraphService};

const COOPERATIVES_QUERY: &str = r#"
query Cooperatives {
  cooperatives(orderBy: registeredAt, orderDirection: desc) {
    id
    coopId
    name
    loanMachine
    active
    registeredAt
  }
}"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CooperativeRow{
    pub id: String,
    #[serde(rename = "coopId")]
    pub coop_id: String,
    pub name: String,
    #[serde(rename = "loanMachine")]
    pub loan_machine: String,
    pub active: bool,
    #[serde(rename = "registeredAt")]
    pub registered_at:String,
}

#[derive(Deserialize)]
struct CooperativesData{
    cooperatives: Vec<CooperativeRow>,
}

pub async fn fetch_cooperatives(
    subgraph: &SubgraphService,
) -> Result<Vec<CooperativeRow>, SubgraphError> {
    // No variables for this query. `()` serializes to `null`, which
    // graph-node accepts. Use an empty struct if you prefer `{}`.
    let data: CooperativesData = subgraph.query(COOPERATIVES_QUERY, ()).await?;
    Ok(data.cooperatives)
}