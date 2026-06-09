use serde::{Serialize, Deserialize};

use crate::services::subgraph::SubgraphError;

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

gql_fetch! {
    pub async fn fetch_cooperatives(subgraph: &SubgraphService)
        -> Result<Vec<CooperativeRow>, SubgraphError>
    {
        query: COOPERATIVES_QUERY,
        vars:  (),
        data:  { cooperatives: Vec<CooperativeRow> },
        map:   |d| Ok(d.cooperatives),
    }
}