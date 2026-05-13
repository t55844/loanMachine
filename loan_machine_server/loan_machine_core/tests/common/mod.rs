// common/mod.rs

pub mod deploy;
pub use deploy::{get_deployed};

use alloy::primitives::Address;
use loan_machine_models::wallet_address::{WalletAddress, from_alloy};

pub fn wa(addr: Address) -> WalletAddress {
    from_alloy(addr)
}