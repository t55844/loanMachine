// loan_machine_core/src/server_logic/loans.rs

use serde::Deserialize;
use crate::services::subgraph::{SubgraphService,SubgraphError};

#[derive(Debug, Deserialize)]
pub struct Loan{
    pub id: String,
    pub amount: String,
    pub borrower: String,
    pub parcels_count: u32,
    pub parcels_pending: u32,
}

#[derive(Deserialize)]
struct LoansResponse {
    loans: Vec<Loan>,
}

const LOANS_FOR_BORROWER_QUERY: &str = r#"
    query LoansForBorrower($borrower: String!) {
        loans(where: { borrower: $borrower }) {
            id
            amount
            borrower
            parcelsCount-json
            parcelsPending
        }
    }
"#;

pub async fn get_loans_for_wallet(
    subgraph: &SubgraphService,
    wallet: &str,
    ) -> Result<Vec<Loan>, SubgraphError> {
        let vars = serde_json::json!({"borrower": wallet.to_lowercase()});
        let response: LoansResponse = subgraph.query(LOANS_FOR_BORROWER_QUERY, vars).await?;
        Ok(response.loans)
    }