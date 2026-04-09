// src/routes/donate.rs
use axum::{extract::State, http::StatusCode, Json};
use alloy::primitives::{Address, FixedBytes, U256};
use crate::{
    config::AppState,
    services::blockchain::BlockchainError,
};
use loan_machine_models::{requests::DonateRequest, responses::TransactionResponse};

pub async fn prepare_donation(
    State(state): State<AppState>,
    Json(req): Json<DonateRequest>,
) -> Result<Json<TransactionResponse>, (StatusCode, String)> {
    let amount: U256 = parse_amount(&req.amount)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let from: Address = req.from
        .parse()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid 'from' address".to_string()))?;

    let coop_id_bytes: FixedBytes<32> = req.coop_id
        .parse()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid coop ID".to_string()))?;

    let factory = &state.blockchain_service.factory;

    // Explicit type: Address
    let loan_machine_addr: Address = factory
        .get_loan_machine(coop_id_bytes)
        .await
        .map_err(|e: BlockchainError| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let data = factory.encode_donate(amount, req.member_id);

    // Explicit type: U256
    let gas_estimate: U256 = factory
        .estimate_donate_gas(loan_machine_addr, amount, req.member_id, from)
        .await
        .map_err(|e: BlockchainError| {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Gas estimation error: {e}"))
        })?;

    Ok(Json(TransactionResponse {
        to:           loan_machine_addr.to_string(),
        data:         format!("0x{}", hex::encode(data.as_ref())),
        value:        "0".to_string(),
        gas_estimate: gas_estimate.to_string(),
    }))
}

fn parse_amount(amount_str: &str) -> Result<U256, String> {
    let amount_float = amount_str
        .parse::<f64>()
        .map_err(|_| "Invalid amount format".to_string())?;
    // USDT has 6 decimals
    let amount_units = (amount_float * 1_000_000.0) as u64;
    Ok(U256::from(amount_units))
}