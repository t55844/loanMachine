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

use crate::services::blockchain::abis::{CoopRegistry, LoanMachine};
use loan_machine_models::responses::{CoopDeployBundle, CoopRegistrationResult};


#[derive(Debug)]
pub enum CoopDeploymentError{
    InvalidAdminAddress(String),
    AdminCountWrong{got: usize, expected: usize},
    FounderNotInAdmins,
    InvalidLoanMachineAddress,
    LoanMachineHasNoCode,
    FounderNotAdminOfDeployedContract,
    Network(String),
    SigninKey(String),
    Encoding(String),
}

impl fmt::Display for CoopDeploymentError{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result{
        match self{
            Self::InvalidAdminAddress(s)             => write!(f, "endereço de admin inválido: {s}"),
            Self::AdminCountWrong { got, expected }  => write!(f, "esperado {expected} admins, recebido {got}"),
            Self::FounderNotInAdmins                 => write!(f, "fundador deve estar entre os admins"),
            Self::InvalidLoanMachineAddress          => write!(f, "endereço do LoanMachine inválido"),
            Self::LoanMachineHasNoCode               => write!(f, "endereço não contém um contrato deployado"),
            Self::FounderNotAdminOfDeployedContract  => write!(f, "fundador não é admin do contrato deployado"),
            Self::Network(s)                         => write!(f, "erro de rede: {s}"),
            Self::SigninKey(s)                      => write!(f, "erro na chave de assinatura: {s}"),
            Self::Encoding(s)                        => write!(f, "erro de encoding: {s}"),

        }
    }
}

impl std::error::Error for CoopDeploymentError{}

pub struct CoopDeploymentService{
    signer: PrivateKeySigner,
    registry_address: Address,
    rpc_url: String,
    loan_machine_bytecode: Vec<u8>,
    usdc_address: Address,
}
use secrecy::{ExposeSecret, SecretString};

impl CoopDeploymentService{
    pub fn new(
        platform_admin_key: SecretString,
        registry_address: &str,
        rpc_url: &str,
        loan_machine_bytecode: Vec<u8>,
        usdc_address: &str,
    ) -> Result<Self, CoopDeploymentError>{
        let signer = PrivateKeySigner::from_str(platform_admin_key.expose_secret().trim_start_matches("0x"))
            .map_err(|e| CoopDeploymentError::SigninKey(e.to_string()))?;

        let registry_address = Address::from_str(registry_address)
            .map_err(|e| CoopDeploymentError::SigninKey(e.to_string()))?;

        let usdc_address = Address::from_str(usdc_address)
            .map_err(|e| CoopDeploymentError::SigninKey(e.to_string()))?;

        Ok(Self{
            signer,
            registry_address,
            rpc_url: rpc_url.to_string(),
            loan_machine_bytecode,
            usdc_address,
        })
    }

    fn generate_access_code() -> String{
        const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
        let mut rng = rand::thread_rng();
        (0..12).map(|_| CHARSET[rng.gen_range(0..CHARSET.len())] as char).collect()
    }

    pub async fn prepare_deploy_bundle(
        &self,
        founder_wallet: &str,
        admin_wallets:  &[String],
        threshold:      u32,
    ) -> Result<CoopDeployBundle , CoopDeploymentError>{

        if admin_wallets.len() != 3{
            return Err(CoopDeploymentError::AdminCountWrong{
                got: admin_wallets.len(),
                expected: 3,
            });
        }

        let founder_addr = Address::from_str(founder_wallet)
            .map_err(|_| CoopDeploymentError::InvalidAdminAddress(founder_wallet.to_string()))?;
    
        let admin_addrs: Result<Vec<Address>, _> = admin_wallets.iter()
            .map(|w| Address::from_str(w)
                .map_err(|_| CoopDeploymentError::InvalidAdminAddress(w.to_string()))
            ).collect();
        let admin_addrs = admin_addrs?;

        if !admin_addrs.contains(&founder_addr){
            return Err(CoopDeploymentError::FounderNotInAdmins);
        }

        let access_code = Self::generate_access_code();

        use alloy::sol_types::SolValue;
        let constructor_args = self.usdc_address.abi_encode();
        let mut deploy_data = self.loan_machine_bytecode.clone();
        deploy_data.extend(&constructor_args);

        let init_call = LoanMachine::initializeMultisigCall{
            admins: admin_addrs,
            threshold: U256::from(threshold),
            accessCode: access_code.clone(),
        };

        let initialize_data = init_call.abi_encode();

        let rpc_url = self.rpc_url
            .parse()
            .map_err(|e| CoopDeploymentError::Network(format!("invalid rpc url: {e}")))?;

        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .on_http(rpc_url);

        let gas_deploy = provider.estimate_gas(&{
            use alloy::rpc::types::TransactionRequest;
            TransactionRequest::default()
                .from(founder_addr)
                .input(deploy_data.clone().into())
        }).await.unwrap_or(3_500_000);

        let gas_initialize = 600_000u64;

        Ok(CoopDeployBundle {
            deploy_data: format!("0x{}", hex::encode(deploy_data)),
            gas_deploy: gas_deploy.to_string(),
            initialize_data: format!("0x{}", hex::encode(initialize_data)),
            gas_initialize: gas_initialize.to_string(),
            access_code,
        })
    }

    pub async fn register_deployed_coop(
        &self,
        name: &str,
        loan_machine_address: &str,
        founder_wallet: &str,
    ) -> Result<CoopRegistrationResult, CoopDeploymentError>{
        let lm_addr = Address::from_str(loan_machine_address)
            .map_err(|_| CoopDeploymentError::InvalidLoanMachineAddress)?;
        let founder_addr = Address::from_str(founder_wallet)
            .map_err(|_| CoopDeploymentError::InvalidAdminAddress(founder_wallet.to_string()))?;

        let rpc_url = self.rpc_url
            .parse()
            .map_err(|e| CoopDeploymentError::Network(format!("invalid rpc url: {e}")))?;


        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(EthereumWallet::from(self.signer.clone()))
            .on_http(rpc_url);
            
        let code = provider.get_code_at(lm_addr).await
            .map_err(|e| CoopDeploymentError::Network(e.to_string()))?;
        if code.is_empty(){
            return Err(CoopDeploymentError::LoanMachineHasNoCode);
        }

        let lm = LoanMachine::new(lm_addr, provider.clone());
        let admins = lm.getAdmins().call().await
            .map_err(|e| CoopDeploymentError::Network(e.to_string()))?
            ._0;
        if !admins.contains(&founder_addr){
            return Err(CoopDeploymentError::FounderNotAdminOfDeployedContract);
        }

        let registry = CoopRegistry::new(self.registry_address, provider);

        let coop_id: FixedBytes<32> = registry
            .registerCoop(name.to_string(), lm_addr)
            .from(self.signer.address())
            .call().await
            .map_err(|e| CoopDeploymentError::Network(e.to_string()))?
            .coopId;

        let pending = registry
            .registerCoop(name.to_string(), lm_addr)
            .from(self.signer.address())
            .send().await
            .map_err(|e| CoopDeploymentError::Network(e.to_string()))?;

        let tx_hash = format!("0x{}", hex::encode(pending.tx_hash()));

        pending.watch().await
            .map_err(|e| CoopDeploymentError::Network(e.to_string()))?;

        Ok(CoopRegistrationResult{
            coop_id_hex: format!("0x{}", hex::encode(coop_id)),
            loan_machine_address: format!("{:?}", lm_addr),
            registration_tx_hash: tx_hash,
        })
    }
}

// Don't leak the key in logs.
impl fmt::Debug for CoopDeploymentService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CoopDeploymentService")
            .field("signer",           &"[REDACTED]")
            .field("registry_address", &self.registry_address)
            .field("rpc_url",          &"[REDACTED]")
            .field("usdc_address",     &self.usdc_address)
            .finish()
    }
}