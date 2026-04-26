// src/models/requests.rs
use serde::{Deserialize, Serialize};


#[derive(Deserialize)]
pub struct VinculateMemberRequest {
    pub member_id:    u32,
    pub wallet_address: String,
    pub coop_id:      String,   // bytes32 hex — which cooperative
    pub access_code:  String,   // cooperative invite code
}

#[derive(Deserialize)]
pub struct DonateRequest {
    pub amount:    String,   // decimal string e.g. "10.50"
    pub member_id: u32,
    pub from:      String,   // sender wallet address
    pub coop_id:   String,   // which cooperative's LoanMachine to donate to
}

#[derive(Serialize, Deserialize,Clone, Copy, PartialEq, Eq, Debug)]
pub enum DocKind{
    Cpf,
    Cnpj,
}

impl DocKind{
    pub fn max_input_len(self) -> usize{
        match self{
            DocKind::Cpf => 14,   // "123.456.789-09"
            DocKind::Cnpj => 18,  // "12.345.678/0001-90"
        }
    }

    pub fn placeholder(self) -> &'static str{
        match self{
            DocKind::Cpf => "000.000.000-00",
            DocKind::Cnpj => "00.000.000/0000-00",
        }
    }

    pub fn label(self) -> &'static str{
        match self{
            DocKind::Cpf => "CPF",
            DocKind::Cnpj => "CNPJ",
        }
    }
}
