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

    pub fn max_digits(&self) -> usize {
        match self {
            DocKind::Cpf  => 11,
            DocKind::Cnpj => 14,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCoopRequest {
    pub name:           String,         
    pub founder_wallet: WalletAddress,         
    pub admin_wallets:  Vec<WalletAddress>,
    pub threshold:      u32,   
    pub doc_kind:    DocKind,
    pub document:    String,         
}
