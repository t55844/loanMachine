// loan_machine_web/tests/common/mod.rs
use std::str::FromStr;
use loan_machine_models::wallet_address::WalletAddress;

/// A syntactically valid wallet address for tests. The bytes are arbitrary
/// (all-1s rather than all-0s so it doesn't collide with Address::ZERO),
/// but they're a real address — meaning the type's validators are satisfied.
pub fn fake_wallet() -> WalletAddress {
    WalletAddress::from_str("0x1111111111111111111111111111111111111111")
        .expect("fake_wallet fixture must parse")
}