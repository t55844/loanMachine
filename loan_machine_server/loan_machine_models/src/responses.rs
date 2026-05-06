// src/models/responses.rs
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CooperativeView {
    pub id: String,
    pub coop_id: String,
    pub name: String,
    pub loan_machine: String,
    pub active: bool,
    pub registered_at: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TransactionResponse {
    pub to:           String,
    pub data:         String,
    pub value:        String,
    pub gas_estimate: String,
}

/// Info about a cooperative instance
#[derive(Serialize, Deserialize, Clone)]
pub struct CoopInfo {
    pub coop_id:      String,   // bytes32 as "0x..." hex string
    pub name:         String,
    pub loan_machine: String,   // contract address
    pub active:       bool,
}

/// Everything the browser needs to submit the vinculation transaction.
/// joinCoop now handles both joining AND vinculation in one tx,
/// so we only need one calldata instead of two.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VinculationBundle {
    pub join_calldata:        String,   // encoded joinCoop(memberId, wallet, accessCode)
    pub loan_machine_address: String,   // target contract for the tx
    pub coop_registry_address:      String,   // coop_registry address (for reference/display)
    pub gas_join:             String,   // gas estimate as string
}

/// Donation transaction data
#[derive(Serialize, Deserialize, Clone)]
pub struct DonationBundle {
    pub approve_calldata:     String,   // USDT.approve(loanMachine, amount)
    pub usdt_address:         String,
    pub donate_calldata:      String,   // LoanMachine.donate(amount, memberId)
    pub loan_machine_address: String,
    pub gas_approve:          String,
    pub gas_donate:           String,
}

/// Bundle returned to the founder for the deploy + initialize flow.
/// The client signs `deploy_tx` first, gets the deployed address,
/// then signs `initialize_calldata` against that address.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoopDeployBundle {
    /// Deployment transaction. `to` is null for deployments —
    /// `data` is bytecode + ABI-encoded constructor args.
    pub deploy_data:        String,    // 0x-prefixed hex
    pub gas_deploy:         String,    // estimated gas 0x-prefixed hex
    /// Calldata for `initializeMultisig(admins, threshold, accessCode)`.
    /// The founder calls this AGAINST the address from the deploy receipt.
    pub initialize_data:    String,
    pub gas_initialize:     String,
    /// Server-generated access code. Shown to founder ONCE.
    /// They must save it before continuing — admins use it to approve members.
    pub access_code:        String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoopRegistrationResult {
    pub coop_id_hex:          String,    // bytes32 returned by registry
    pub loan_machine_address: String,
    pub registration_tx_hash: String,
}