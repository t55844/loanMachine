mod common;
use crate::common::deploy::DeployedEnv;
use loan_machine_core::server_logic::user_profile::get_user_profile_logic;
use loan_machine_core::services::subgraph::SubgraphService;
use loan_machine_models::wallet_address::from_alloy;

use serde_json::json;
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Build the JSON shape returned by `fetch_user_related_coops` for "user is
/// related to env.coop via {membership, approval_request}".
fn coop_entry(env: &DeployedEnv) -> serde_json::Value {
    json!({
        "cooperative": {
            "id":          env.loan_machine_address.to_lowercase(),
            "coopId":      env.coop_id_hex,
            "name":        "Test Coop",
            "loanMachine": env.loan_machine_address.to_lowercase(),
            "active":      true,
        }
    })
}

#[tokio::test]
async fn empty_profile_when_subgraph_returns_no_relations() {
    let env = common::get_deployed().await;

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_string_contains("UserRelatedCoops"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": { "asMember": [], "asProposer": [] }
        })))
        .mount(&server).await;

    let subgraph = SubgraphService::new(server.uri());
    let p = get_user_profile_logic(&subgraph, &env.blockchain, from_alloy(env.unapproved_wallet))
        .await.unwrap();

    assert!(p.coops.is_empty());
}

#[tokio::test]
async fn founder_is_admin_member_and_approved() {
    let env = common::get_deployed().await;

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_string_contains("UserRelatedCoops"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": { "asMember": [coop_entry(&env)], "asProposer": [] }
        })))
        .mount(&server).await;
    // Founder is admin → count_actionable_proposals fires PendingForUser
    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_string_contains("PendingForUser"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "created": [], "executed": [], "confirmedByMe": [], "cosigned": [],
            }
        })))
        .mount(&server).await;

    let subgraph = SubgraphService::new(server.uri());
    let p = get_user_profile_logic(&subgraph, &env.blockchain, from_alloy(env.approved_wallet))
        .await.unwrap();

    assert_eq!(p.coops.len(), 1);
    let c = &p.coops[0];
    assert!( c.is_admin);
    assert!(!c.is_moderator);
    assert!( c.is_member, "founder vinculated at init");
    assert!( c.is_approved);
    assert!(!c.has_pending_approval, "founder is already a member");
    assert_eq!(c.pending_count, 0);
}

#[tokio::test]
async fn stranger_with_pending_proposal_shows_pending_approval_flag() {
    let env = common::get_deployed().await;

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_string_contains("UserRelatedCoops"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": { "asMember": [], "asProposer": [coop_entry(&env)] }
        })))
        .mount(&server).await;
    // Stranger is neither admin nor moderator → count_actionable_proposals
    // short-circuits before hitting the subgraph; no PendingForUser mock needed.

    let subgraph = SubgraphService::new(server.uri());
    let p = get_user_profile_logic(&subgraph, &env.blockchain, from_alloy(env.unapproved_wallet))
        .await.unwrap();

    assert_eq!(p.coops.len(), 1);
    let c = &p.coops[0];
    assert!(!c.is_admin);
    assert!(!c.is_moderator);
    assert!(!c.is_member);
    assert!(!c.is_approved);
    assert!( c.has_pending_approval);
    assert_eq!(c.pending_count, 0);
}

#[tokio::test]
async fn admin_pending_count_excludes_already_confirmed_proposals() {
    let env = common::get_deployed().await;

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_string_contains("UserRelatedCoops"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": { "asMember": [coop_entry(&env)], "asProposer": [] }
        })))
        .mount(&server).await;

    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_string_contains("PendingForUser"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "created":       [{"proposalId": "0"}, {"proposalId": "1"}, {"proposalId": "2"}],
                "executed":      [{"proposalId": "2"}],            // proposal 2 done
                "confirmedByMe": [{"proposalId": "0"}],            // viewer already acted on 0
                "cosigned":      [],
            }
        })))
        .mount(&server).await;

    let subgraph = SubgraphService::new(server.uri());
    let p = get_user_profile_logic(&subgraph, &env.blockchain, from_alloy(env.approved_wallet))
        .await.unwrap();

    assert_eq!(p.coops[0].pending_count, 1, "only proposal 1 is actionable for the viewer");
}

#[tokio::test]
async fn pending_count_is_zero_for_non_capable_wallet_even_with_open_proposals() {
    // Sanity guard: if you ever wire pending_count from a different path,
    // strangers must keep getting 0.  This test pins that.
    let env = common::get_deployed().await;

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_string_contains("UserRelatedCoops"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": { "asMember": [], "asProposer": [coop_entry(&env)] }
        })))
        .mount(&server).await;

    // Note: no PendingForUser mock. If the code regresses and queries it for
    // a stranger, the request will hit an unmocked wiremock and the test
    // fails — that's the assertion.

    let subgraph = SubgraphService::new(server.uri());
    let p = get_user_profile_logic(&subgraph, &env.blockchain, from_alloy(env.unapproved_wallet))
        .await.unwrap();

    assert_eq!(p.coops[0].pending_count, 0);
}