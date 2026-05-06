use alloy::primitives::{Address, FixedBytes};
use loan_machine_models::responses::{CoopInfo, VinculationBundle};
use loan_machine_models::requests::{DocKind};

use crate::services::blockchain::BlockchainService;
use crate::services::identity::IdentityService;
use crate::services::blockchain::contract_errors::friendly_from_error;


#[derive(Debug, thiserror::Error)]
pub enum VinculationLogicError {
    #[error("Endereço de carteira inválido")]
    ServerLogicInvalidWalletAddress,
    #[error("ID de cooperativa inválido")]
    ServerLogicInvalidCoopId,
    #[error("identitiy{0}")]
    ServerLogicIdentity(String),
    #[error("blockchain error: {0}")]
    ServerLogicBlockchain(String),
    #[error("carteira não aprovada")]
    ServerLogicWalletNotApproved,
}

pub async fn get_wallet_coop_logic(
    blockchain: &BlockchainService,
    smart_wallet: &str,
) -> Result<Option<CoopInfo>, VinculationLogicError> {
    let wallet: Address = smart_wallet
        .parse()
        .map_err(|_| VinculationLogicError::ServerLogicInvalidWalletAddress)?;
    
    let coop_id = blockchain.coop_registry
        .get_wallet_coop(wallet)
        .await
        .map_err(|e| VinculationLogicError::ServerLogicBlockchain(e.to_string()))?;

    if coop_id == FixedBytes::<32>::ZERO{
        return Ok(None);
    }    

    let info = blockchain.coop_registry
        .get_coop_info(coop_id)
        .await
        .map_err(|e| VinculationLogicError::ServerLogicBlockchain(e.to_string()))?;

    Ok(Some(info))
}


pub async fn prepare_first_vinculation_logic(
    identity: &IdentityService,
    blockchain: &BlockchainService,
    coop_registry_address: &str,
    doc_kind: DocKind,
    document: String,
    smart_wallet: String,
    coop_id: String,
    access_code: String,
) -> Result<VinculationBundle, VinculationLogicError> {
eprintln!("eprintln: enter the prepare_first_vinculation_logic");

    let member_id = match doc_kind{
        DocKind::Cpf => identity.cpf_to_member_id(&document),
        DocKind::Cnpj => identity.cnpj_to_member_id(&document),
    }
    .map_err(|e| VinculationLogicError::ServerLogicIdentity(e.to_string()))?;
eprintln!("get member id correctly: {}",member_id);

    let wallet: Address = smart_wallet
        .parse()
        .map_err(|_| VinculationLogicError::ServerLogicInvalidWalletAddress)?;
eprintln!("get wallet  correctly: {}",wallet);

    let coop_id_bytes: FixedBytes<32> = coop_id
        .parse()
        .map_err(|_| VinculationLogicError::ServerLogicInvalidCoopId)?;
eprintln!("get coop_id_bytes correctly: {}",coop_id_bytes);

    let coop_registry = &blockchain.coop_registry;
eprintln!("get coop_registry correctly: ");

    let loan_machine_addr = coop_registry.get_loan_machine(coop_id_bytes).await
        .map_err(|e| VinculationLogicError::ServerLogicBlockchain(e.to_string()))?;
eprintln!("get loan_machine_addr correctly: ");


    let approved = coop_registry.is_wallet_approved(loan_machine_addr, wallet).await
    .map_err(|e| VinculationLogicError::ServerLogicBlockchain(friendly_from_error(&e)))?;

    if !approved{
        return Err(VinculationLogicError::ServerLogicWalletNotApproved);
    }

    let join_calldata = coop_registry.encode_join_coop(member_id, wallet, &access_code);
eprintln!("[vinc] calling joinCoop (estimating gas)");

    let gas = coop_registry
        .estimate_join_coop_gas(loan_machine_addr, member_id, wallet, &access_code)
        .await
        .map_err(|e| VinculationLogicError::ServerLogicBlockchain(e.to_string()))?;

    Ok(VinculationBundle{
        join_calldata: format!("0x{}", hex::encode(join_calldata.as_ref())),
        loan_machine_address: loan_machine_addr.to_string(),
        coop_registry_address: coop_registry_address.to_string(),
        gas_join: format!("0x{:x}", gas),
    })
}