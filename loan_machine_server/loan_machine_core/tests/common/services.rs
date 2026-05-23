//! Constructors for the services that every chain-touching test needs.

use std::str::FromStr;

use alloy::signers::local::PrivateKeySigner;
use secrecy::SecretString;

use loan_machine_core::services::coop_deployment::CoopDeploymentService;
use loan_machine_core::services::identity::IdentityService;

use crate::common::deploy::DeployedEnv;

pub fn make_deployment_service(env: &DeployedEnv) -> CoopDeploymentService {
    CoopDeploymentService::new(
        SecretString::from(env.platform_admin_key_hex.clone()),
        &env.coop_registry_address,
        &env.rpc_url,
        env.loan_machine_bytecode.clone(),
        &env.usdt_address,
    )
    .expect("CoopDeploymentService::new")
}

/// IdentityService from the `COOP_SALT` env var — same as prod.
pub fn make_identity_from_env() -> IdentityService {
    IdentityService::from_env()
        .expect("COOP_SALT must be set — add it to .env or your test environment")
}

/// The platform-admin signer (admin1) reconstructed from the deploy env.
pub fn platform_admin_signer(env: &DeployedEnv) -> PrivateKeySigner {
    PrivateKeySigner::from_str(env.platform_admin_key_hex.trim_start_matches("0x"))
        .expect("valid platform admin key")
}