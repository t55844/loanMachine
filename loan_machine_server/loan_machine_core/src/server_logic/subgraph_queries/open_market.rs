use serde::{Deserialize, Serialize};

use crate::services::subgraph::{SubgraphError, SubgraphService};

// status_in: [0, 1] fetches only Pending (0) and PartiallyCovered (1) loans.
// The subgraph now tracks status through all lifecycle events:
//   LoanCovered  → status 1 (partial) or 2 (full)
//   LoanFunded   → status 3 (active)
//   LoanCompleted→ status 4 (repaid)
//   LoanRequisitionCreatedCancelled(6) → status 6 (cancelled)
// currentCoverage and creationTimestamp are also maintained by the subgraph,
// so no on-chain calls are needed to display the open market.
const QUERY: &str = r#"
query OpenMarket($coopId: String!) {
  loanRequisitionCreatedCancelledEvents(
    where: { cooperative: $coopId, status_in: [0, 1] }
    orderBy: blockTimestamp
    orderDirection: asc
    first: 100
  ) {
    requisitionId
    borrower
    amount
    parcelsCount
    currentCoverage
    creationTimestamp
  }
}"#;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Vars<'a> {
    coop_id: &'a str,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Row {
    requisition_id:     String,
    borrower:           String,
    amount:             String,
    parcels_count:      u32,
    current_coverage:   u32,
    creation_timestamp: String, // BigInt → quoted string in GraphQL JSON
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Data {
    loan_requisition_created_cancelled_events: Vec<Row>,
}

pub struct RawOpenRequisition {
    pub requisition_id:     String,
    pub borrower:           String,
    pub amount:             String,
    pub parcels_count:      u32,
    pub current_coverage:   u32,
    pub creation_timestamp: String,
}

pub async fn fetch_open_requisitions(
    s:       &SubgraphService,
    coop_id: &str,
) -> Result<Vec<RawOpenRequisition>, SubgraphError> {
    let vars = Vars { coop_id };
    let data: Data = s.query(QUERY, vars).await?;

    Ok(data
        .loan_requisition_created_cancelled_events
        .into_iter()
        .map(|r| RawOpenRequisition {
            requisition_id:     r.requisition_id,
            borrower:           r.borrower,
            amount:             r.amount,
            parcels_count:      r.parcels_count,
            current_coverage:   r.current_coverage,
            creation_timestamp: r.creation_timestamp,
        })
        .collect())
}
