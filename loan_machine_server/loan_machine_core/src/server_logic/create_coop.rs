// loan_machine_core/src/server_logic/create_coop.rs
use thiserror::Error;
use loan_machine_models::wallet_address::{WalletAddress, to_alloy};
use loan_machine_models::responses::{CoopDeployBundle, CoopRegistrationResult};
use alloy::primitives::Address;
use crate::services::coop_deployment::{CoopDeploymentService, CoopDeploymentError};
use loan_machine_models::requests::DocKind;
use crate::services::identity::{IdentityService, IdentityError};

#[derive(Debug, Error)]
pub enum CreateCoopLogicError {
    #[error("nome da cooperativa inválido")]
    InvalidName,
    #[error("endereço da LoanMachine inválido")]
    InvalidLoanMachineAddress,
    #[error(transparent)]
    Deployment(#[from] CoopDeploymentError),
    #[error(transparent)]
    ServerLogicIdentity(#[from] IdentityError),
}

pub async fn prepare_create_coop_logic(
    identity: &IdentityService,
    deployment: &CoopDeploymentService,
    name: String,
    founder_wallet: WalletAddress,
    admin_wallets:  Vec<WalletAddress>,   // ← was Vec<String>
    threshold: u32,
    doc_kind: DocKind,
    document: String,
) -> Result<CoopDeployBundle, CreateCoopLogicError> {
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed.len() > 100 {
        return Err(CreateCoopLogicError::InvalidName);
    }

    let member_id = match doc_kind {
        DocKind::Cpf  => identity.cpf_to_member_id(&document),
        DocKind::Cnpj => identity.cnpj_to_member_id(&document),
    }?;

    // ── BOUNDARY: app domain → chain domain ──
    let founder_addr = to_alloy(&founder_wallet);
    let admin_addrs: Vec<Address> = admin_wallets.iter().map(to_alloy).collect();
    // ── from here on, only chain vocabulary ──

    let bundle = deployment
        .prepare_deploy_bundle(founder_addr, &admin_addrs, threshold, member_id,)
        .await?;
    Ok(bundle)
}

pub async fn register_deployed_coop_logic(
    deployment: &CoopDeploymentService,
    name: String,
    loan_machine_address: String,         // ← parsed here
    founder_wallet: WalletAddress,
) -> Result<CoopRegistrationResult, CreateCoopLogicError> {
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed.len() > 100 {
        return Err(CreateCoopLogicError::InvalidName);
    }

    let lm_addr: Address = loan_machine_address.parse()
        .map_err(|_| CreateCoopLogicError::InvalidLoanMachineAddress)?;
    let founder_addr = to_alloy(&founder_wallet);

    deployment
        .register_deployed_coop(trimmed, lm_addr, founder_addr)
        .await
        .map_err(Into::into)
}

