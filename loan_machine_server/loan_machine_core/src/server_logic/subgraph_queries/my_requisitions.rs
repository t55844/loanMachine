use serde::{Deserialize, Serialize};

use crate::services::subgraph::{SubgraphError, SubgraphService};

const QUERY: &str = r#"
query MyRequisitions($coopId: String!, $borrower: Bytes!) {
  loanRequisitionCreatedCancelledEvents(
    where: { cooperative: $coopId, borrower: $borrower }
    orderBy: blockTimestamp
    orderDirection: desc
    first: 20
  ) {
    requisitionId
    amount
    parcelsCount
    status
  }
}"#;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Vars<'a> {
    coop_id:  &'a str,
    borrower: &'a str,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Row {
    requisition_id: String,
    amount:         String,
    parcels_count:  u32,
    status:         u8,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Data {
    loan_requisition_created_cancelled_events: Vec<Row>,
}

pub struct RawRequisition {
    pub requisition_id: String,
    pub amount:         String,
    pub parcels_count:  u32,
    pub status:         u8,
}

pub async fn fetch_my_requisitions(
    s:        &SubgraphService,
    coop_id:  &str,
    borrower: &str,
) -> Result<Vec<RawRequisition>, SubgraphError> {
    let vars = Vars { coop_id, borrower };
    let data: Data = s.query(QUERY, vars).await?;

    Ok(data
        .loan_requisition_created_cancelled_events
        .into_iter()
        .map(|r| RawRequisition {
            requisition_id: r.requisition_id,
            amount:         r.amount,
            parcels_count:  r.parcels_count,
            status:         r.status,
        })
        .collect())
}
