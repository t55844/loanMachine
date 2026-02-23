use serde::{Serialize,Deserialize};

#[derive(Debug, Serialize,Deserialize,Clone)]
pub struct TransactionResponse {
    pub to: String,          // contract address
    pub data: String,        // hex encoded calldata
    pub value: String,       // "0" (if no ETH sent)
    pub gas_estimate: String, // estimated gas as decimal string
}