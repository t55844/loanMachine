use alloy::primitives::{Address};
use thiserror::Error;

use loan_machine_models::responses::MemberFinancials;
use loan_machine_models::wallet_address::{to_alloy, WalletAddress};

use crate::services::blockchain::{ BlockchainError, BlockchainService};
use crate::services::subgraph::SubgraphService;

use crate::server_logic::helpers::{resolve_member_id, resolve_member_reputation};
use crate::server_logic::subgraph_queries::member_financials::fetch_member_financials;

#[derive(Debug, Error)]
pub enum MemberFinancialsError {
    #[error("invalid coop id: {0}")]
    InvalidCoopId(String),

    #[error("wallet is not a member of this cooperative")]
    WalletNotMember,

    #[error(transparent)]
    Blockchain(#[from] BlockchainError),

    #[error("on-chain lastBorrowTime does not fit in u64: {0}")]
    InvalidLastBorrowTime(String),
}

pub async fn get_member_financials_logic(
    subgraph:    &SubgraphService,
    blockchain:  &BlockchainService,
    coop_id_hex: &str,
    wallet:      WalletAddress,
) -> Result<MemberFinancials, MemberFinancialsError> {
    let wallet_addr: Address = to_alloy(&wallet);

    let (_coop_id_b32, loan_machine_addr, provider, contract) = coop_context!(
        blockchain:      blockchain,
        coop_id_hex:     coop_id_hex,
        invalid_coop_id: MemberFinancialsError::InvalidCoopId(coop_id_hex.into()),
        contract,
    );


    // Security gate: the wallet must be an active member of this coop.
    let member_id = resolve_member_id(subgraph, provider.as_ref(), loan_machine_addr, &wallet)
        .await?
        .ok_or(MemberFinancialsError::WalletNotMember)?;
    let member_id_hex = format!("0x{}", hex::encode(member_id.as_slice()));

    // Reputation: subgraph-first, chain fallback.
    let reputation = resolve_member_reputation(subgraph, provider.as_ref(), loan_machine_addr, member_id)
        .await?;

    // Financial snapshot from chain.  This is always needed for
    // inCoverage / withdrawable / allowance which the subgraph cannot provide.
    let chain_f = contract.getUserFinancials(wallet_addr).call().await
        .map_err(BlockchainError::from_call)?;

    // Try subgraph for donation / borrowing / last_borrow_time.
    // On success those values replace the chain equivalents; on any subgraph
    // error the chain values are used as-is.
    let coop_id_str = format!("{loan_machine_addr:#x}").to_lowercase();
    let wallet_str  = wallet.to_string();

    let chain_last_borrow_time: u64 = chain_f.lastBorrowTime.try_into()
        .map_err(|e| MemberFinancialsError::InvalidLastBorrowTime(format!("{e}")))?;

    let (donation, borrowing, last_borrow_time) =
        match fetch_member_financials(subgraph, &coop_id_str, &wallet_str, &member_id_hex).await {
            Ok(s) => (
                s.donation.unwrap_or_else(|| chain_f.donation.to_string()),
                s.borrowing.unwrap_or_else(|| chain_f.borrowing.to_string()),
                s.last_borrow_time.unwrap_or(chain_last_borrow_time),
            ),
            Err(_) => (
                chain_f.donation.to_string(),
                chain_f.borrowing.to_string(),
                chain_last_borrow_time,
            ),
        };

    Ok(MemberFinancials {
        wallet:            wallet.to_string(),
        member_id:         member_id_hex,
        reputation,
        donation,
        borrowing,
        last_borrow_time,
        in_coverage:       chain_f.inCoverage.to_string(),
        withdrawable:      chain_f.withdrawable.to_string(),
        allowance:         chain_f.allowance.to_string(),
        loan_machine_address: format!("{loan_machine_addr:#x}"),
    })
}
