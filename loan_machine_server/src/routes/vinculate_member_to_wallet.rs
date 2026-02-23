//vinculate_member_to_wallet.rs
use axum::{
    extract::State,
    http::StatusCode,
    Json
};

use alloy::primitives::Address;
use crate::{
    config::AppState,
    models::{requests::VinculateMemberRequest, responses::TransactionResponse},
};


pub async fn prepare_vinculation_to_wallet(
    State(state): State<AppState>, 
    Json(req): Json<VinculateMemberRequest>) 
    -> Result<Json<TransactionResponse>, (StatusCode, String)> {
        let member_id: u32 = req.member_id;
        let wallet_address: Address = req.wallet_address.parse().map_err(|_| (StatusCode::BAD_REQUEST, "Invalid wallet address".to_string()))?;

        let bc = &state.blockchain_service;
        let data = bc.encode_vinculation_member(member_id, wallet_address);

        let gas_estimate = bc.estimate_vinculation_member_to_wallet_gas(member_id, wallet_address)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Gas estimation error: {}", e)))?;

        let response: TransactionResponse = TransactionResponse{
            to: state.contract_address.0.clone(),
            data: format!("0x{}", hex::encode(data.as_ref())),
            value: "0".to_string(),
            gas_estimate: gas_estimate.to_string()
        };

        Ok(Json(response))
}