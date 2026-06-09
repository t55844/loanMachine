use alloy::primitives::{ FixedBytes};
use loan_machine_models::responses::{CoopInfo, VinculationBundle,};
use loan_machine_models::requests::DocKind;
use loan_machine_models::wallet_address::{WalletAddress,to_alloy};
use crate::services::blockchain::{BlockchainService, BlockchainError};
use crate::services::identity::{IdentityService, IdentityError}; // ← swap IdentityError for the real name
use crate::services::blockchain::loan_machine_fns_helpers::{is_wallet_approved, encode_join_coop, estimate_join_coop_gas};
use crate::services::subgraph::SubgraphService;
use crate::server_logic::helpers::resolve_wallet_coop;

#[derive(Debug, thiserror::Error)]
pub enum VinculationLogicError {
    #[error("Invalid wallet address")]
    ServerLogicInvalidWalletAddress,

    #[error("Invalid cooperative ID")]
    ServerLogicInvalidCoopId,

    #[error("wallet not approved")]
    ServerLogicWalletNotApproved,

    #[error(transparent)]
    ServerLogicIdentity(#[from] IdentityError),

    #[error(transparent)]
    ServerLogicBlockchain(#[from] BlockchainError),


}
pub async fn get_wallet_coop_logic(
    subgraph: &SubgraphService,
    blockchain: &BlockchainService,
    smart_wallet: WalletAddress,
) -> Result<Option<CoopInfo>, VinculationLogicError> {
    Ok(resolve_wallet_coop(subgraph, blockchain, &smart_wallet).await?)
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

    let approved = is_wallet_approved(&coop_registry.provider, loan_machine_addr, wallet_addr).await?;
    if !approved {
        return Err(VinculationLogicError::ServerLogicWalletNotApproved);
    }


    let join_calldata = encode_join_coop( member_id, wallet_addr, &access_code);

    let gas = estimate_join_coop_gas(
        &coop_registry.provider,
        loan_machine_addr, member_id,
        wallet_addr, &access_code)
        .await?;

    Ok(VinculationBundle {
        join_calldata: format!("0x{}", hex::encode(join_calldata.as_ref())),
        loan_machine_address:  loan_machine_addr.to_string(),
        coop_registry_address: coop_registry_address.to_string(),
        gas_join: format!("0x{:x}", gas),
    })
}


