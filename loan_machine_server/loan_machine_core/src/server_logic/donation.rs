use alloy::primitives::{Address, U256};
use thiserror::Error;

use loan_machine_models::responses::DonationBundle;
use loan_machine_models::wallet_address::{to_alloy, WalletAddress};

use crate::services::blockchain::{BlockchainError, BlockchainService};
use crate::services::blockchain::loan_machine_fns_helpers::{
    encode_approve, estimate_approve_gas, encode_donate, get_erc20_balance,
};
use crate::services::subgraph::SubgraphService;

use crate::server_logic::helpers::resolve_member_id;

#[derive(Debug, Error)]
pub enum DonationLogicError {
    #[error("invalid coop id: {0}")] InvalidCoopId(String),
    #[error("invalid amount: {0}")] InvalidAmount(String),
    #[error("wallet is not a member of this cooperative")] WalletNotMember,
    #[error("insufficient USDC/USDT balance: have {available}, need {required}")]
    InsufficientBalance { available: String, required: String },
    #[error(transparent)] Blockchain(#[from] BlockchainError),
}

/// `donate()` calls `usdtToken.transferFrom`, which reverts with
/// "insufficient allowance" if simulated before the `approve` tx (signed
/// in the same bundle) has landed. Gas can't be estimated on-chain at
/// bundle-prep time, so use a fixed ceiling — `donate()` is just a
/// transferFrom plus a few storage writes and events.
const GAS_DONATE_FALLBACK: u64 = 150_000;

pub async fn donate_logic(
    subgraph:     &SubgraphService,
    blockchain:   &BlockchainService,
    coop_id_hex:  &str,
    usdc_address: &str,
    wallet:       WalletAddress,
    amount:       String,
) -> Result<DonationBundle, DonationLogicError> {
    let amount: U256 = amount.parse()
        .map_err(|_| DonationLogicError::InvalidAmount(amount))?;

    let wallet_addr: Address = to_alloy(&wallet);

    let (_coop_id_b32, loan_machine_addr, provider, _contract) = coop_context!(
        blockchain:      blockchain,
        coop_id_hex:     coop_id_hex,
        invalid_coop_id: DonationLogicError::InvalidCoopId(coop_id_hex.into()),
        contract,
    );

    let member_id = resolve_member_id(subgraph, provider.as_ref(), loan_machine_addr, &wallet)
        .await?
        .ok_or(DonationLogicError::WalletNotMember)?;

    let usdc_addr: Address = usdc_address.parse()
        .map_err(|_| DonationLogicError::Blockchain(BlockchainError::InvalidAddress))?;

    let balance = get_erc20_balance(provider.as_ref(), usdc_addr, wallet_addr).await?;
    if balance < amount {
        return Err(DonationLogicError::InsufficientBalance {
            available: balance.to_string(),
            required:  amount.to_string(),
        });
    }

    let approve_calldata = encode_approve(loan_machine_addr, amount);
    let gas_approve = estimate_approve_gas(
        provider.as_ref(), usdc_addr, loan_machine_addr, amount, wallet_addr,
    ).await?;

    let donate_calldata = encode_donate(amount, member_id);

    Ok(DonationBundle {
        approve_calldata:     format!("0x{}", hex::encode(approve_calldata.as_ref())),
        usdt_address:         usdc_addr.to_string(),
        donate_calldata:      format!("0x{}", hex::encode(donate_calldata.as_ref())),
        loan_machine_address: loan_machine_addr.to_string(),
        gas_approve:          format!("0x{:x}", gas_approve),
        gas_donate:           format!("0x{GAS_DONATE_FALLBACK:x}"),
    })
}
