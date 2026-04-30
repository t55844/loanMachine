// loan_machine_core/src/server_logic/create_coop.rs

use crate::services::coop_deployment::{ CoopDeploymentService};
use loan_machine_models::responses::{CoopDeployBundle, CoopRegistrationResult};

#[derive(Debug)]
pub enum CreateCoopLogicError{
    ServerLogicInvalidName,
    ServerLogicDeployment(String),
}

impl std::fmt::Display for CreateCoopLogicError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result{
        match self{
            Self::ServerLogicInvalidName     => write!(f, "nome da cooperativa inválido"),
            Self::ServerLogicDeployment(s)   => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for CreateCoopLogicError{}

pub async fn prepare_create_coop_logic(
    deployment: &CoopDeploymentService,
    name: String,
    founder_wallet: String,
    admin_wallets: Vec<String>,
    threshold: u32,
) -> Result<CoopDeployBundle, CreateCoopLogicError>{
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed.len() > 100{
        return Err(CreateCoopLogicError::ServerLogicInvalidName);
    }

    deployment
        .prepare_deploy_bundle(&founder_wallet, &admin_wallets, threshold)
        .await
        .map_err(|e| CreateCoopLogicError::ServerLogicDeployment(e.to_string()))
}

pub async fn register_deployed_coop_logic(
    deployment: &CoopDeploymentService,
    name: String,
    loan_machine_address: String,
    founder_wallet: String,
) -> Result<CoopRegistrationResult, CreateCoopLogicError>{
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed.len() > 100{
        return Err(CreateCoopLogicError::ServerLogicInvalidName);
    }

    deployment
        .register_deployed_coop(trimmed, &loan_machine_address, &founder_wallet)
        .await
        .map_err(|e| CreateCoopLogicError::ServerLogicDeployment(e.to_string()))
}