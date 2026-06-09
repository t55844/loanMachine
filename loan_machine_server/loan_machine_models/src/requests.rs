// src/models/requests.rs
use serde::{Deserialize, Serialize};
use crate::wallet_address::WalletAddress;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterDeployedCoopRequest {
    pub name:                 String,
    pub loan_machine_address: String,
    pub founder_wallet:       WalletAddress,
}


#[derive(Deserialize)]
pub struct VinculateMemberRequest {
    pub member_id:    u32,
    pub wallet_address: WalletAddress,
    pub coop_id:      String,   // bytes32 hex — which cooperative
    pub access_code:  String,   // cooperative invite code
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCoopRequest {
    pub name:           String,
    pub founder_wallet: WalletAddress,
    pub admin_wallets:  Vec<WalletAddress>,
    pub threshold:      u32,
}
