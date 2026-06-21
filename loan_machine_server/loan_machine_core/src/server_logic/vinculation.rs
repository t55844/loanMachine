use loan_machine_models::responses::{CoopInfo, VinculationBundle};
use loan_machine_models::wallet_address::{WalletAddress, to_alloy};
use crate::services::blockchain::{BlockchainService, BlockchainError};
use crate::services::identity::IdentityService;
use crate::services::blockchain::loan_machine_fns_helpers::{is_wallet_approved, encode_join_coop, estimate_join_coop_gas};
use crate::services::subgraph::SubgraphService;
use crate::server_logic::helpers::resolve_wallet_coop;

#[derive(Debug, thiserror::Error)]
pub enum VinculationLogicError {
    #[error("Invalid wallet address")]
    ServerLogicInvalidWalletAddress,

    #[error("Invalid cooperative ID")]
    InvalidCoopId(String),

    #[error("wallet not approved")]
    ServerLogicWalletNotApproved,

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
    smart_wallet: WalletAddress,
    coop_id: String,
) -> Result<VinculationBundle, VinculationLogicError> {
    let member_id = identity.wallet_to_member_id(smart_wallet.as_bytes());
    let wallet_addr = to_alloy(&smart_wallet);

    let (_coop_id_b32, loan_machine_addr, provider, _contract) = coop_context!(
        blockchain:      blockchain,
        coop_id_hex:     coop_id,
        invalid_coop_id: VinculationLogicError::InvalidCoopId(coop_id.into()),
        contract,
    );

    let approved = is_wallet_approved(provider.as_ref(), loan_machine_addr, wallet_addr).await?;
    if !approved {
        return Err(VinculationLogicError::ServerLogicWalletNotApproved);
    }

    let join_calldata = encode_join_coop(member_id, wallet_addr);

    let gas = estimate_join_coop_gas(
        provider.as_ref(),
        loan_machine_addr, member_id,
        wallet_addr)
        .await?;

    Ok(VinculationBundle {
        join_calldata: format!("0x{}", hex::encode(join_calldata.as_ref())),
        loan_machine_address:  loan_machine_addr.to_string(),
        coop_registry_address: coop_registry_address.to_string(),
        gas_join: format!("0x{:x}", gas),
    })
}
