// src/models/requests.rs

#[derive(serde::Deserialize)]
pub struct VinculateMemberRequest {
    pub member_id:    u32,
    pub wallet_address: String,
    pub coop_id:      String,   // bytes32 hex — which cooperative
    pub access_code:  String,   // cooperative invite code
}

#[derive(serde::Deserialize)]
pub struct DonateRequest {
    pub amount:    String,   // decimal string e.g. "10.50"
    pub member_id: u32,
    pub from:      String,   // sender wallet address
    pub coop_id:   String,   // which cooperative's LoanMachine to donate to
}