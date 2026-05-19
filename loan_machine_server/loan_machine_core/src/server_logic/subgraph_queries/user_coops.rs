// loan_machine_core/src/server_logic/subgraph_queries/user_coops.rs
//

use std::collections::HashSet;

use serde::Deserialize;

use loan_machine_models::responses::UserCoop;
use crate::services::subgraph::{SubgraphError, SubgraphService};

const USER_COOPS_QUERY: &str = r#"
  query UserCoops($wallet: Bytes!) {
    memberRegisteredEvents(where: { wallet: $wallet }) {
      cooperative {
        id
        coopId
        name
        loanMachine
        active
      }
    }
  }
"#;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawResponse {
    member_registered_events: Vec<RawEntry>,
}

#[derive(Debug, Deserialize)]
struct RawEntry {
    cooperative: UserCoop,
}

pub async fn get_user_coops_logic(
    s:      &SubgraphService,
    wallet: &str,
) -> Result<Vec<UserCoop>, SubgraphError> {
    let variables = serde_json::json!({
        "wallet": wallet.to_lowercase(),
    });

    let raw: RawResponse = s.query(USER_COOPS_QUERY, variables).await?;

    let mut seen = HashSet::new();
    let coops = raw
        .member_registered_events
        .into_iter()
        .map(|e| e.cooperative)
        .filter(|c| seen.insert(c.id.clone()))
        .collect();

    Ok(coops)
}