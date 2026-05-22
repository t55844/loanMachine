//! Compile-time pinning of server-fn signatures.
//! These don't *run* — they fail at compile if a signature drifts.

#![allow(dead_code, unused_imports)]

use super::admin_approvals::{
    list_pending_approvals, prepare_confirm_proposal, prepare_cosign_proposal,
};
use loan_machine_models::responses::{ApproveWalletPending, RequestApprovalBundle};

#[allow(unused)]
async fn _list_signature_pin(coop_id: String)
    -> Result<ApproveWalletPending, leptos::server_fn::ServerFnError>
{
    list_pending_approvals(coop_id).await
}

#[allow(unused)]
async fn _confirm_signature_pin(coop_id: String, proposal_id: u64)
    -> Result<RequestApprovalBundle, leptos::server_fn::ServerFnError>
{
    prepare_confirm_proposal(coop_id, proposal_id).await
}

#[allow(unused)]
async fn _cosign_signature_pin(coop_id: String, proposal_id: u64)
    -> Result<RequestApprovalBundle, leptos::server_fn::ServerFnError>
{
    prepare_cosign_proposal(coop_id, proposal_id).await
}