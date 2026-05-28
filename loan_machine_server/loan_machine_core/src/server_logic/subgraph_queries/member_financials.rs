// Subgraph-first financial profile for a single coop member.
//
// Queries the four event streams that reflect balance changes
// (DonatedEvent, WithdrawnEvent, BorrowedEvent, RepaidEvent) plus the
// latest ReputationChangedEvent.  The most-recent timestamp wins between
// competing events for the same balance (e.g. a repayment supersedes the
// prior borrow row).
//
// Fields the subgraph cannot provide (inCoverage, withdrawable, allowance)
// must be fetched from the chain via getUserFinancials — the caller handles
// those regardless of whether this query succeeds.

use serde::{Deserialize, Serialize};
use crate::services::subgraph::{SubgraphError, SubgraphService};

const QUERY: &str = r#"
query MemberFinancials($coopId: String!, $wallet: Bytes!, $memberId: Bytes!) {
  latestDonate: donatedEvents(
    where: { cooperative: $coopId, donor: $wallet }
    orderBy: blockTimestamp
    orderDirection: desc
    first: 1
  ) { totalDonation blockTimestamp }

  latestWithdraw: withdrawnEvents(
    where: { cooperative: $coopId, donor: $wallet }
    orderBy: blockTimestamp
    orderDirection: desc
    first: 1
  ) { donations blockTimestamp }

  latestBorrow: borrowedEvents(
    where: { cooperative: $coopId, borrower: $wallet }
    orderBy: blockTimestamp
    orderDirection: desc
    first: 1
  ) { totalBorrowing blockTimestamp }

  latestRepay: repaidEvents(
    where: { cooperative: $coopId, borrower: $wallet }
    orderBy: blockTimestamp
    orderDirection: desc
    first: 1
  ) { remainingDebt blockTimestamp }

  latestReputation: reputationChangedEvents(
    where: { cooperative: $coopId, memberId: $memberId }
    orderBy: blockTimestamp
    orderDirection: desc
    first: 1
  ) { newReputation }
}"#;

#[derive(Serialize)]
struct Vars<'a> {
    #[serde(rename = "coopId")]   coop_id:   &'a str,
                                  wallet:    &'a str,
    #[serde(rename = "memberId")] member_id: &'a str,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DonateRow     { total_donation: String,  block_timestamp: String }
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WithdrawRow   { donations:      String,  block_timestamp: String }
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BorrowRow     { total_borrowing: String, block_timestamp: String }
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RepayRow      { remaining_debt:  String, block_timestamp: String }
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReputationRow { new_reputation:  i32 }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Data {
    latest_donate:     Vec<DonateRow>,
    latest_withdraw:   Vec<WithdrawRow>,
    latest_borrow:     Vec<BorrowRow>,
    latest_repay:      Vec<RepayRow>,
    latest_reputation: Vec<ReputationRow>,
}

pub struct SubgraphMemberFinancials {
    /// Most-recent donation balance derived from event history. None if no events.
    pub donation:         Option<String>,
    /// Most-recent borrowing balance derived from event history. None if no events.
    pub borrowing:        Option<String>,
    /// blockTimestamp of the latest BorrowedEvent (proxy for lastBorrowTime). None if never borrowed.
    pub last_borrow_time: Option<u64>,
    /// Most-recent known reputation. None if no ReputationChangedEvent exists yet.
    pub reputation:       Option<i32>,
}

pub async fn fetch_member_financials(
    s:         &SubgraphService,
    coop_id:   &str,
    wallet:    &str,
    member_id: &str,
) -> Result<SubgraphMemberFinancials, SubgraphError> {
    let vars = Vars { coop_id, wallet: &wallet.to_lowercase(), member_id };
    let data: Data = s.query(QUERY, vars).await?;

    // Pick the most-recent balance between DonatedEvent and WithdrawnEvent.
    let donation = most_recent_str(
        data.latest_donate.first().map(|r| (r.total_donation.as_str(), r.block_timestamp.as_str())),
        data.latest_withdraw.first().map(|r| (r.donations.as_str(), r.block_timestamp.as_str())),
    );

    // Pick the most-recent balance between BorrowedEvent and RepaidEvent.
    let borrow_row = data.latest_borrow.first();
    let repay_row  = data.latest_repay.first();

    let more_recent_is_borrow = match (&borrow_row, &repay_row) {
        (Some(b), Some(r)) => b.block_timestamp >= r.block_timestamp,
        (Some(_), None)    => true,
        _                  => false,
    };
    let borrowing = if more_recent_is_borrow {
        borrow_row.map(|r| r.total_borrowing.clone())
    } else {
        repay_row.map(|r| r.remaining_debt.clone())
    };

    let last_borrow_time = borrow_row
        .and_then(|r| r.block_timestamp.parse::<u64>().ok());

    let reputation = data.latest_reputation.first().map(|r| r.new_reputation);

    Ok(SubgraphMemberFinancials { donation, borrowing, last_borrow_time, reputation })
}

fn most_recent_str<'a>(
    a: Option<(&'a str, &'a str)>,
    b: Option<(&'a str, &'a str)>,
) -> Option<String> {
    match (a, b) {
        (None, None)               => None,
        (Some((v, _)), None)       => Some(v.to_string()),
        (None, Some((v, _)))       => Some(v.to_string()),
        (Some((va, ta)), Some((vb, tb))) => {
            if ta >= tb { Some(va.to_string()) } else { Some(vb.to_string()) }
        }
    }
}
