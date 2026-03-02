// src/routes/vinculate_member_to_wallet.rs
use axum::{extract::State, http::StatusCode, Json};
use alloy::primitives::{Address, FixedBytes, U256};
use crate::{
    config::AppState,
    models::{requests::VinculateMemberRequest, responses::TransactionResponse},
    services::blockchain::BlockchainError,
};

pub async fn prepare_vinculation_to_wallet(
    State(state): State<AppState>,
    Json(req): Json<VinculateMemberRequest>,
) -> Result<Json<TransactionResponse>, (StatusCode, String)> {
    let member_id: u32 = req.member_id;

    let wallet_address: Address = req.wallet_address
        .parse()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid wallet address".to_string()))?;

    let coop_id_bytes: FixedBytes<32> = req.coop_id
        .parse()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid coop ID".to_string()))?;

    let factory = &state.blockchain_service.factory;

    // Explicit type: Address
    let loan_machine_addr: Address = factory
        .get_loan_machine(coop_id_bytes)
        .await
        .map_err(|e: BlockchainError| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let data = factory.encode_join_coop(member_id, wallet_address, &req.access_code);

    // Explicit type: U256
    let gas_estimate: U256 = factory
        .estimate_join_coop_gas(
            loan_machine_addr,
            member_id,
            wallet_address,
            &req.access_code,
        )
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