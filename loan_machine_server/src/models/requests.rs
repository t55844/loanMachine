use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DonateRequest {
    pub amount: String,
    pub member_id: u32,
    pub from: String
}

#[derive(Debug, Deserialize)]
pub struct VinculateMemberRequest {
    pub member_id: u32,
    pub wallet_address: String
}