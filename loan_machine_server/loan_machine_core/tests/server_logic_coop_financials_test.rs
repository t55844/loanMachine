mod common;

use loan_machine_core::server_logic::coop_financials::{
    get_coop_financials_logic, CoopFinancialsError,
};

#[tokio::test]
async fn happy_path_returns_financials_for_coop() {
    let env = common::get_deployed().await;

    let result = get_coop_financials_logic(&env.blockchain, &env.coop_id_hex)
        .await
        .expect("deployed coop should return financials");

    assert!(result.is_active, "freshly deployed coop should be active");
    assert!(result.active_member_count >= 1, "founder should count as an active member");

    for amount in [&result.total_donations, &result.total_borrowed, &result.available_balance, &result.contract_balance] {
        amount.parse::<u128>().expect("amount fields must be decimal strings");
    }
}

#[tokio::test]
async fn invalid_coop_id_returns_error() {
    let env = common::get_deployed().await;

    let result = get_coop_financials_logic(&env.blockchain, "not-bytes32").await;

    assert!(
        matches!(result, Err(CoopFinancialsError::InvalidCoopId(_))),
        "expected InvalidCoopId, got {:?}", result,
    );
}
