// src/models/responses.rs

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct TransactionResponse {
    pub to:           String,
    pub data:         String,
    pub value:        String,
    pub gas_estimate: String,
}

/// Info about a cooperative instance
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CoopInfo {
    pub coop_id:      String,   // bytes32 as "0x..." hex string
    pub name:         String,
    pub loan_machine: String,   // contract address
    pub active:       bool,
}

/// Everything the browser needs to submit the vinculation transaction.
/// joinCoop now handles both joining AND vinculation in one tx,
/// so we only need one calldata instead of two.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct VinculationBundle {
    pub join_calldata:        String,   // encoded joinCoop(memberId, wallet, accessCode)
    pub loan_machine_address: String,   // target contract for the tx
    pub factory_address:      String,   // factory address (for reference/display)
    pub gas_join:             String,   // gas estimate as string
}

/// Donation transaction data
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct DonationBundle {
    pub approve_calldata:     String,   // USDT.approve(loanMachine, amount)
    pub usdt_address:         String,
    pub donate_calldata:      String,   // LoanMachine.donate(amount, memberId)
    pub loan_machine_address: String,
    pub gas_approve:          String,
    pub gas_donate:           String,
}