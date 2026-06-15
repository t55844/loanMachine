use alloy::primitives::{Address, U256};
use thiserror::Error;

use loan_machine_models::responses::WithdrawalBundle;
use loan_machine_models::wallet_address::{to_alloy, WalletAddress};

use crate::services::blockchain::{BlockchainError, BlockchainService};
use crate::services::blockchain::loan_machine_fns_helpers::{
    encode_withdraw, estimate_withdraw_gas,
};
use crate::services::subgraph::SubgraphService;

use crate::server_logic::helpers::resolve_member_id;

#[derive(Debug, Error)]
pub enum WithdrawalLogicError {
    #[error("invalid coop id: {0}")] InvalidCoopId(String),
    #[error("invalid amount: {0}")] InvalidAmount(String),
    #[error("wallet is not a member of this cooperative")] WalletNotMember,
    #[error("insufficient withdrawable balance: have {available}, requested {requested}")]
    InsufficientBalance { available: String, requested: String },
    #[error(transparent)] Blockchain(#[from] BlockchainError),
}

/// `withdraw()` transfers `amount` of the member's donation balance back to
/// their wallet immediately — no delay/request step. The withdrawable
/// balance (`donation - inCoverage`) is checked here, before building the
/// calldata, so the client gets a clear error instead of a contract revert.
pub async fn withdraw_logic(
    subgraph:    &SubgraphService,
    blockchain:  &BlockchainService,
    coop_id_hex: &str,
    wallet:      WalletAddress,
    amount:      String,
) -> Result<WithdrawalBundle, WithdrawalLogicError> {
    let amount: U256 = amount.parse()
        .map_err(|_| WithdrawalLogicError::InvalidAmount(amount))?;

    let wallet_addr: Address = to_alloy(&wallet);

    let (_coop_id_b32, loan_machine_addr, provider, contract) = coop_context!(
        blockchain:      blockchain,
        coop_id_hex:     coop_id_hex,
        invalid_coop_id: WithdrawalLogicError::InvalidCoopId(coop_id_hex.into()),
        contract,
    );

    let member_id = resolve_member_id(subgraph, provider.as_ref(), loan_machine_addr, &wallet)
        .await?
        .ok_or(WithdrawalLogicError::WalletNotMember)?;

    let chain_f = contract.getUserFinancials(wallet_addr).call().await
        .map_err(BlockchainError::from_call)?;

    if amount > chain_f.withdrawable {
        return Err(WithdrawalLogicError::InsufficientBalance {
            available: chain_f.withdrawable.to_string(),
            requested: amount.to_string(),
        });
    }

    let withdraw_calldata = encode_withdraw(amount, member_id);
    let gas_withdraw = estimate_withdraw_gas(
        provider.as_ref(), loan_machine_addr, amount, member_id, wallet_addr,
    ).await?;

    Ok(WithdrawalBundle {
        withdraw_calldata:    format!("0x{}", hex::encode(withdraw_calldata.as_ref())),
        loan_machine_address: loan_machine_addr.to_string(),
        gas_withdraw:         format!("0x{:x}", gas_withdraw),
    })
}
