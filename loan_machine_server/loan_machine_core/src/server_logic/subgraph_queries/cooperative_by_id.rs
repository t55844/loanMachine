use serde::Serialize;

use crate::services::subgraph::SubgraphError;
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
#[serde(rename_all = "camelCase")]
struct Vars<'a> {
    coop_id: &'a str,
}

gql_fetch! { 
    pub async fn fetch_one_coop(subgraph: &SubgraphService, coop_id: &str)
        -> Result<Option<CooperativeRow>, SubgraphError>
    {
        query: QUERY,
        vars:  Vars { coop_id },
        data:  { cooperatives: Vec<CooperativeRow> },
        map:   |d| Ok(d.cooperatives.into_iter().next()),
    }
}