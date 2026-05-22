use alloy::primitives::{Address, B256};
use thiserror::Error;

use loan_machine_models::responses::{UserProfile, UserProfileCoop};
use loan_machine_models::wallet_address::{to_alloy, WalletAddress};

use crate::services::blockchain::{abis::LoanMachine, BlockchainError, BlockchainService};
use crate::services::subgraph::{SubgraphError, SubgraphService};

use crate::server_logic::subgraph_queries::user_related_coops::fetch_user_related_coops;
use crate::server_logic::subgraph_queries::pending_proposals_count::count_actionable_proposals;
use crate::server_logic::helpers::resolve_member_id;

#[derive(Debug, Error)]
pub enum UserProfileError {
    #[error("subgraph error: {0}")] Subgraph(#[from] SubgraphError),
    #[error(transparent)]           Blockchain(#[from] BlockchainError),
    #[error("malformed address: {0}")] BadAddress(String),
}

pub async fn get_user_profile_logic(
    subgraph:   &SubgraphService,
    blockchain: &BlockchainService,
    wallet:     WalletAddress,
) -> Result<UserProfile, UserProfileError> {
    let wallet_hex  = format!("{wallet}");
    let wallet_addr = to_alloy(&wallet);

    let related = fetch_user_related_coops(subgraph, &wallet_hex).await?;

    let provider = &blockchain.coop_registry.provider;
    let mut out  = Vec::with_capacity(related.len());

    for c in related {
        let lm_addr: Address = c.loan_machine.parse()
            .map_err(|_| UserProfileError::BadAddress(c.loan_machine.clone()))?;
        let contract = LoanMachine::new(lm_addr, provider.clone());

        let is_admin = contract.isAdmin(wallet_addr).call().await
            .map_err(BlockchainError::from_call)?._0;

        let member_id_b32 = resolve_member_id( 
            &subgraph, &provider, lm_addr, &wallet
            ).await?
            .unwrap_or(B256::ZERO);
        let is_member = member_id_b32 != B256::ZERO;

        let is_moderator = if is_member {
            contract.isModerator(member_id_b32).call().await
                .map_err(BlockchainError::from_call)?._0
        } else { false };

        let member_id = member_id_b32.to_string();

        let is_approved = if is_member { true } else {
            contract.isWalletApproved(wallet_addr).call().await
                .map_err(BlockchainError::from_call)?._0
        };

        let has_pending_approval = c.via_approval_request && !is_approved;

        let pending_count = count_actionable_proposals(
            subgraph, lm_addr, wallet_addr, is_admin, is_moderator
        ).await?;

        out.push(UserProfileCoop {
            id: c.id, coop_id: c.coop_id, name: c.name,
            loan_machine: c.loan_machine, active: c.active,
            member_id, is_member, is_admin, is_moderator,
            is_approved, has_pending_approval,
            pending_count,
        });
    }

    Ok(UserProfile { wallet: wallet_hex, coops: out })
}