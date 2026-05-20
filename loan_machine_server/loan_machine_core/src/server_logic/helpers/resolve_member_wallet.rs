// loan_machine_core/src/server_logic/helpers/resolve_member_wallet.rs

use alloy::primitives::{Address, B256};
use serde::{Deserialize, Serialize};

use loan_machine_models::wallet_address::{from_alloy, WalletAddress};
use crate::services::subgraph::SubgraphService;

const QUERY: &str = r#"
query WalletByMemberId($coopId: String!, $memberId: Bytes!) {
  memberRegisteredEvents(
    where: { cooperative: $coopId, memberId: $memberId, isFirstWallet: true }
    first: 1
  ) {
    wallet
  }
}"#;

#[derive(Serialize)]
struct Vars<'a> {
    #[serde(rename = "coopId")]   coop_id:   String,
    #[serde(rename = "memberId")] member_id: &'a str,
}

#[derive(Deserialize)]
struct Row { wallet: String }

#[derive(Deserialize)]
struct Data {
    #[serde(rename = "memberRegisteredEvents")]
    events: Vec<Row>,
}

/// Subgraph-only: the contract has no `memberId → wallet` getter.
/// Returns None on miss or any subgraph failure — caller decides the fallback.
pub async fn resolve_member_wallet(
    subgraph:          &SubgraphService,
    loan_machine_addr: Address,
    member_id:         B256,
) -> Option<WalletAddress> {
    let coop_id    = format!("{loan_machine_addr:#x}").to_lowercase();
    let member_hex = format!("0x{}", hex::encode(member_id.as_slice()));
    let vars       = Vars { coop_id, member_id: &member_hex };

    let data: Data    = subgraph.query(QUERY, vars).await.ok()?;
    let row           = data.events.into_iter().next()?;
    let addr: Address = row.wallet.parse().ok()?;
    Some(from_alloy(addr))
}