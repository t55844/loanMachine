#![allow(dead_code, unused_imports)]

pub mod deploy;
pub mod subgraph_mock;
pub mod payloads;
pub mod services;
pub mod proposals;
pub mod elections;
pub mod financials;

pub use deploy::get_deployed;
pub use subgraph_mock::{mount_query, server_returning, server_returning_status};
pub use payloads::{
    cooperative_minimal, cooperative_row, cooperatives_with, coop_entry_for,
    empty_payload, graphql_error_payload, member_events_empty, member_events_with,
    one_cooperative_payload,
};
pub use services::{make_deployment_service, make_identity_from_env, platform_admin_signer};
pub use proposals::{
    confirm_as_second_admin, confirm_as_third_admin,
    propose_add_admin, propose_wallet_approval, propose_fresh_wallet_approval,
};
pub use elections::{
    bootstrap_admin1_as_moderator, vinculate_second_admin, admin2_member_id,
    open_election, cast_vote_as_second_admin, bootstrap_active_election,
};
pub use financials::{donate_as, withdraw_as, mint_usdt};

use alloy::primitives::Address;
use loan_machine_models::wallet_address::{from_alloy, WalletAddress};

pub fn wa(addr: Address) -> WalletAddress {
    from_alloy(addr)
}

/// Parse `0x...` as u64 or panic. Most tests just want the value.
pub fn parse_hex_u64(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16)
        .unwrap_or_else(|e| panic!("bad hex value {s:?}: {e}"))
}

/// Boolean form for wire-format tests where the assertion *is* "this parses".
pub fn is_hex_u64(s: &str) -> bool {
    s.strip_prefix("0x")
        .and_then(|hex| u64::from_str_radix(hex, 16).ok())
        .is_some()
}