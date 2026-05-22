use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::services::subgraph::{SubgraphError, SubgraphService};

const QUERY: &str = r#"
query UserRelatedCoops($wallet: Bytes!) {
  asMember: memberRegisteredEvents(where: { wallet: $wallet }) {
    cooperative { id coopId name loanMachine active }
  }
  asProposer: proposalCreatedEvents(where: { proposer: $wallet, pType: 2 }) {
    cooperative { id coopId name loanMachine active }
  }
}"#;

#[derive(Serialize)]
struct Vars<'a> { wallet: &'a str }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CoopRow {
    id:           String,
    coop_id:      String,
    name:         String,
    loan_machine: String,
    active:       bool,
}

#[derive(Debug, Deserialize)]
struct Entry { cooperative: CoopRow }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Data {
    as_member:   Vec<Entry>,
    as_proposer: Vec<Entry>,
}

pub struct RelatedCoop {
    pub id:                   String,
    pub coop_id:              String,
    pub name:                 String,
    pub loan_machine:         String,
    pub active:               bool,
    pub via_membership:       bool,
    pub via_approval_request: bool,
}

pub async fn fetch_user_related_coops(
    s:      &SubgraphService,
    wallet: &str,
) -> Result<Vec<RelatedCoop>, SubgraphError> {
    let vars = Vars { wallet: &wallet.to_lowercase() };
    let data: Data = s.query(QUERY, vars).await?;

    let mut map: HashMap<String, RelatedCoop> = HashMap::new();

    for e in data.as_member {
        let c = e.cooperative;
        map.entry(c.id.clone()).or_insert(RelatedCoop {
            id: c.id, coop_id: c.coop_id, name: c.name,
            loan_machine: c.loan_machine, active: c.active,
            via_membership: false, via_approval_request: false,
        }).via_membership = true;
    }

    for e in data.as_proposer {
        let c = e.cooperative;
        map.entry(c.id.clone()).or_insert(RelatedCoop {
            id: c.id, coop_id: c.coop_id, name: c.name,
            loan_machine: c.loan_machine, active: c.active,
            via_membership: false, via_approval_request: false,
        }).via_approval_request = true;
    }

    Ok(map.into_values().collect())
}