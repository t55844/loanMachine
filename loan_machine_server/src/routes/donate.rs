//donate.rs
use axum::{
    extract::State,
    http::StatusCode,
    Json
};

use alloy::primitives::{Address, U256};
use crate::{
    config::AppState,
    models::{requests::DonateRequest, responses::TransactionResponse},  
};

pub async fn prepare_donation(
    State(state): State<AppState>,
    Json(req): Json<DonateRequest>,
) -> Result<Json<TransactionResponse>, (StatusCode, String)> {
    // 1. Parse inputs
    let amount = parse_amount(&req.amount).map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    let from: Address = req.from.parse().map_err(|_| (StatusCode::BAD_REQUEST, "Invalid 'from' address".to_string()))?;
    
    // 2. Create blockchain service ( keep a persistent instance in AppState to avoid reconnecting)
    let bc = &state.blockchain_service;
    
    // 3. Encode transaction data
    let data = bc.encode_donate(amount, req.member_id);

    // 4. Estimate gas
    let gas_estimate = bc.estimate_donate_gas(amount, req.member_id, from)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Gas estimation error: {}", e)))?;

    // 5. Build response
    let response = TransactionResponse{
        to: state.contract_address.0.clone(),
        data: format!("0x{}", hex::encode(data.as_ref())),
        value: "0".to_string(),
        gas_estimate: gas_estimate.to_string()
    };

    Ok(Json(response))
}

fn parse_amount(amount_str: &str) -> Result<U256, String> {
    let amount_float = amount_str.parse::<f64>().map_err(|_| "Invalid amount format")?;
    let amount_wei = (amount_float * 1_000_000.0) as u64;

    Ok(U256::from(amount_wei))
}