use alloy::primitives::{ FixedBytes};
use loan_machine_models::responses::{CoopInfo, VinculationBundle,};
use loan_machine_models::requests::DocKind;
use loan_machine_models::wallet_address::{WalletAddress,to_alloy};
use crate::services::blockchain::{BlockchainService, BlockchainError};
use crate::services::identity::{IdentityService, IdentityError}; // ← swap IdentityError for the real name

#[derive(Debug, thiserror::Error)]
pub enum VinculationLogicError {
    #[error("Endereço de carteira inválido")]
    ServerLogicInvalidWalletAddress,

    #[error("ID de cooperativa inválido")]
    ServerLogicInvalidCoopId,

    #[error("carteira não aprovada")]
    ServerLogicWalletNotApproved,

    #[error(transparent)]
    ServerLogicIdentity(#[from] IdentityError),

    #[error(transparent)]
    ServerLogicBlockchain(#[from] BlockchainError),
}
pub async fn get_wallet_coop_logic(
    blockchain: &BlockchainService,
    smart_wallet: WalletAddress,
) -> Result<Option<CoopInfo>, VinculationLogicError> {
    let wallet_addr = to_alloy(&smart_wallet);   // boundary
    let coop_id = blockchain.coop_registry.get_wallet_coop(wallet_addr).await?;
    if coop_id == FixedBytes::<32>::ZERO {
        return Ok(None);
    }
    let info = blockchain.coop_registry.get_coop_info(coop_id).await?;
    Ok(Some(info))
}

pub async fn prepare_first_vinculation_logic(
    identity: &IdentityService,
    blockchain: &BlockchainService,
    coop_registry_address: &str,
    doc_kind: DocKind,
    document: String,
    smart_wallet: WalletAddress,
    coop_id: String,
    access_code: String,
) -> Result<VinculationBundle, VinculationLogicError> {
    let member_id = match doc_kind {
        DocKind::Cpf  => identity.cpf_to_member_id(&document),
        DocKind::Cnpj => identity.cnpj_to_member_id(&document),
    }?;

    let wallet_addr  = to_alloy(&smart_wallet);   // boundary
    let coop_id_bytes: FixedBytes<32> = coop_id.parse()
        .map_err(|_| VinculationLogicError::ServerLogicInvalidCoopId)?;

    let coop_registry     = &blockchain.coop_registry;
    let loan_machine_addr = coop_registry.get_loan_machine(coop_id_bytes).await?;

    let approved = coop_registry
        .is_wallet_approved(loan_machine_addr, wallet_addr).await?;
    if !approved {
        return Err(VinculationLogicError::ServerLogicWalletNotApproved);
    }

    let join_calldata = coop_registry
        .encode_join_coop(member_id, wallet_addr, &access_code);
    let gas = coop_registry
        .estimate_join_coop_gas(loan_machine_addr, member_id, wallet_addr, &access_code)
        .await?;

    Ok(VinculationBundle {
        join_calldata: format!("0x{}", hex::encode(join_calldata.as_ref())),
        loan_machine_address:  loan_machine_addr.to_string(),
        coop_registry_address: coop_registry_address.to_string(),
        gas_join: format!("0x{:x}", gas),
    })
}