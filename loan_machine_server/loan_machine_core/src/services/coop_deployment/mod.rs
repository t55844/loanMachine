// loan_machine_core/src/services/coop_deployment/mod.rs
//
// CoopDeploymentService
//
// Holds the platform admin signer key — the only key authorized to call
// CoopRegistry.registerCoop. Two responsibilities:
//
//   1. Prepare a deploy bundle for the founder's Privy wallet to sign.
//   2. Register a deployed LoanMachine in the registry, but only after
//      verifying that the requesting founder is actually one of its admins.
//
// The private key NEVER leaves this struct. Custom Debug redacts it.
// AppState clones via Arc — no key duplication in memory.
 
use std::fmt;
use std::str::FromStr;
 
use alloy::network::EthereumWallet;
use alloy::primitives::{Address, FixedBytes, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::local::PrivateKeySigner;
use alloy::sol_types::SolCall;
use rand::Rng;
use secrecy::{ExposeSecret, SecretString};
use thiserror::Error;

use crate::services::blockchain::abis::{CoopRegistry, LoanMachine};
use crate::services::blockchain::contract_errors::{ extract_revert_data, translate_revert};
use loan_machine_models::responses::{CoopDeployBundle, CoopRegistrationResult};


// ── Constants ────────────────────────────────────────────────────────────
//
// Pulled out of magic-number-land so tests can reference them by name and
// changes are obvious in code review.
 
/// Multisig admin count. The contract expects exactly this many addresses.
const REQUIRED_ADMIN_COUNT: usize = 3;
 
/// Length of the human-shareable access code printed for cooperative founders.
/// 12 chars from a 32-char alphabet → ~60 bits of entropy. Shrinking this
/// reduces entropy; growing it hurts UX. Locked by
/// `deploy_bundle_access_code_is_human_friendly_length`.
const ACCESS_CODE_LEN: usize = 12;
 
/// Confusable-stripped alphabet (no I/O/0/1).
const ACCESS_CODE_CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
 
/// Gas fallback when `eth_estimateGas` fails (e.g. unfunded deployer).
/// Production-safe but noisy in tests — see
/// `deploy_gas_estimate_is_realistic_not_fallback`.
const GAS_DEPLOY_FALLBACK: u64 = 3_500_000;
 
/// Hardcoded gas for `initializeMultisig`. We can't simulate it (the contract
/// doesn't exist yet at bundle-prep time), so we set a generous ceiling.
const GAS_INITIALIZE_LIMIT: u64 = 600_000;

#[derive(Debug, Error)]
pub enum CoopDeploymentError{
     #[error("admin addess invalid: {0}")]
    InvalidAdminAddress(Address),
 
    #[error("expected {expected} admins, got {got}")]
    AdminCountWrong { got: usize, expected: usize },
 
    #[error("founder must be between admins")]
    FounderNotInAdmins,
 
    #[error("LoanMachine address invalid")]
    InvalidLoanMachineAddress,
 
    #[error("addres don't has an deployed contract")]
    LoanMachineHasNoCode,
 
    #[error("founder ins't an contract deployed")]
    FounderNotAdminOfDeployedContract,
 
    /// Bad address in service configuration (registry, USDC). Distinct from
    /// `InvalidAdminAddress` which is bad user input.
    #[error("econfigured address invalid: {field}")]
    InvalidConfiguredAddress { field: &'static str },
 
    #[error("net error: {0}")]
    Network(String),
 
    #[error("signature key error: {0}")]
    SigningKey(String),

    #[error("contract call failed")]
    Call(#[source] alloy::contract::Error),
    
    #[error("transport error")]
    Transport(#[from] alloy::transports::TransportError),

    #[error("transaction confirmation failed")]
    PendingTx(#[from] alloy::providers::PendingTransactionError),

    #[error("{message}")]
    ContractRevert {
        message: String,
        #[source]
        source: alloy::contract::Error,
    },
 
}

impl CoopDeploymentError{
    pub fn from_call(err: alloy::contract::Error) -> Self{
        if let Some(bytes) = extract_revert_data(&err) {
            Self::ContractRevert { 
                message: translate_revert(&bytes), source: err, 
                }
        } else {
            Self::Call(err)
        }
        
    }
}


// ── Service ──────────────────────────────────────────────────────────────
 
pub struct CoopDeploymentService {
    signer: PrivateKeySigner,
    registry_address: Address,
    rpc_url: String,
    loan_machine_bytecode: Vec<u8>,
    usdc_address: Address,
}


impl CoopDeploymentService {
    pub fn new(
        platform_admin_key: SecretString,
        registry_address: &str,
        rpc_url: &str,
        loan_machine_bytecode: Vec<u8>,
        usdc_address: &str,
    ) -> Result<Self, CoopDeploymentError> {
        let signer = PrivateKeySigner::from_str(
            platform_admin_key.expose_secret().trim_start_matches("0x"),
        )
        .map_err(|e| CoopDeploymentError::SigningKey(e.to_string()))?;
 
        let registry_address = Address::from_str(registry_address)
            .map_err(|_| CoopDeploymentError::InvalidConfiguredAddress { field: "registry" })?;
 
        let usdc_address = Address::from_str(usdc_address)
            .map_err(|_| CoopDeploymentError::InvalidConfiguredAddress { field: "usdc" })?;
 
        Ok(Self {
            signer,
            registry_address,
            rpc_url: rpc_url.to_string(),
            loan_machine_bytecode,
            usdc_address,
        })
    }

    fn generate_access_code() -> String {
        let mut rng = rand::thread_rng();
        (0..ACCESS_CODE_LEN)
            .map(|_| ACCESS_CODE_CHARSET[rng.gen_range(0..ACCESS_CODE_CHARSET.len())] as char)
            .collect()
    }

       // Inside CoopDeploymentService impl

    pub async fn prepare_deploy_bundle(
        &self,
        founder_addr: Address,
        admin_addrs: &[Address],         // already alloy
        threshold: u32,
    ) -> Result<CoopDeployBundle, CoopDeploymentError> {
        if admin_addrs.len() != REQUIRED_ADMIN_COUNT {
            return Err(CoopDeploymentError::AdminCountWrong {
                got: admin_addrs.len(),
                expected: REQUIRED_ADMIN_COUNT,
            });
        }
        if !admin_addrs.contains(&founder_addr) {
            return Err(CoopDeploymentError::FounderNotInAdmins);
        }

        let access_code = Self::generate_access_code();

        use alloy::sol_types::SolValue;
        let constructor_args = self.usdc_address.abi_encode();
        let mut deploy_data = self.loan_machine_bytecode.clone();
        deploy_data.extend(&constructor_args);

        let init_call = LoanMachine::initializeMultisigCall {
            admins:     admin_addrs.to_vec(),
            threshold:  U256::from(threshold),
            accessCode: access_code.clone(),
        };
        let initialize_data = init_call.abi_encode();

        let rpc_url = self.rpc_url.parse()
            .map_err(|e| CoopDeploymentError::Network(format!("invalid rpc url: {e}")))?;
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .on_http(rpc_url);

        let gas_deploy = provider
            .estimate_gas(&{
                use alloy::rpc::types::TransactionRequest;
                TransactionRequest::default()
                    .from(founder_addr)
                    .input(deploy_data.clone().into())
            })
            .await
            .unwrap_or(GAS_DEPLOY_FALLBACK);

        Ok(CoopDeployBundle {
            deploy_data:    format!("0x{}", hex::encode(deploy_data)),
            gas_deploy:     format!("0x{:x}", gas_deploy),
            initialize_data: format!("0x{}", hex::encode(initialize_data)),
            gas_initialize: format!("0x{:x}", GAS_INITIALIZE_LIMIT),
            access_code,
        })
    }

    pub async fn register_deployed_coop(
        &self,
        name: &str,
        loan_machine_addr: Address,     // ← was &str + Address::from_str
        founder_addr: Address,
    ) -> Result<CoopRegistrationResult, CoopDeploymentError> {
        let rpc_url = self.rpc_url.parse()
            .map_err(|e| CoopDeploymentError::Network(format!("invalid rpc url: {e}")))?;
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(EthereumWallet::from(self.signer.clone()))
            .on_http(rpc_url);

        let registry_code = provider.get_code_at(self.registry_address).await?;
        if registry_code.is_empty() {
            return Err(CoopDeploymentError::Network(format!(
                "CoopRegistry not deployed at {}", self.registry_address
            )));
        }
        let lm_code = provider.get_code_at(loan_machine_addr).await?;
        if lm_code.is_empty() {
            return Err(CoopDeploymentError::LoanMachineHasNoCode);
        }

        let lm = LoanMachine::new(loan_machine_addr, provider.clone());
        let admins = lm.getAdmins().call().await
            .map_err(CoopDeploymentError::from_call)?._0;
        if !admins.contains(&founder_addr) {
            return Err(CoopDeploymentError::FounderNotAdminOfDeployedContract);
        }

        let registry = CoopRegistry::new(self.registry_address, provider);
        let coop_id: FixedBytes<32> = registry
            .registerCoop(name.to_string(), loan_machine_addr)
            .from(self.signer.address())
            .call().await
            .map_err(CoopDeploymentError::from_call)?
            .coopId;

        let pending = registry
            .registerCoop(name.to_string(), loan_machine_addr)
            .from(self.signer.address())
            .send().await
            .map_err(CoopDeploymentError::from_call)?;

        let tx_hash = format!("0x{}", hex::encode(pending.tx_hash()));
        pending.watch().await?;

        Ok(CoopRegistrationResult {
            coop_id_hex: format!("0x{}", hex::encode(coop_id)),
            loan_machine_address: format!("{:?}", loan_machine_addr),
            registration_tx_hash: tx_hash,
        })
    }
}
 
// Don't leak the key in logs.
impl fmt::Debug for CoopDeploymentService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CoopDeploymentService")
            .field("signer", &"[REDACTED]")
            .field("registry_address", &self.registry_address)
            .field("rpc_url", &"[REDACTED]")
            .field("usdc_address", &self.usdc_address)
            .finish()
    }
}
 