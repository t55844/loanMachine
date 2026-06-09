use super::admin_panel::{action_button_label, viewer_has_acted, ApprovalAction};
use loan_machine_models::responses::ApproveWalletProposalRow;

fn row(viewer_confirmed: bool, moderator_cosigned: bool) -> ApproveWalletProposalRow {
    ApproveWalletProposalRow {
        proposal_id:        0,
        proposer:           "0x0000000000000000000000000000000000000000".into(),
        created_at:         0,
        confirmations:      0,
        moderator_cosigned,
        viewer_confirmed,
    }
}

#[test]
fn admin_button_idle_label() {
    assert_eq!(action_button_label(ApprovalAction::AdminConfirm,    false), "Confirm");
    assert_eq!(action_button_label(ApprovalAction::ModeratorCosign, false), "Co-sign");
}

#[test]
fn sending_label_is_action_independent() {
    assert_eq!(action_button_label(ApprovalAction::AdminConfirm,    true), "Sending…");
    assert_eq!(action_button_label(ApprovalAction::ModeratorCosign, true), "Sending…");
}

#[test]
fn admin_acted_iff_viewer_confirmed() {
    assert!( viewer_has_acted(&row(true,  false), ApprovalAction::AdminConfirm));
    assert!(!viewer_has_acted(&row(false, true),  ApprovalAction::AdminConfirm));
}

#[test]
fn moderator_acted_iff_moderator_cosigned() {
    // Note: this *is* shared state across all moderators — once anyone cosigns
    // it's done. That's by contract design; the test pins the intent.
    assert!( viewer_has_acted(&row(false, true),  ApprovalAction::ModeratorCosign));
    assert!(!viewer_has_acted(&row(true,  false), ApprovalAction::ModeratorCosign));
}

#[test]
fn the_two_actions_are_orthogonal() {
    // An admin-also-moderator sees both panels; the same row should report
    // "acted" independently for each capability.
    let r = row(/*viewer_confirmed*/ true, /*moderator_cosigned*/ false);
    assert!( viewer_has_acted(&r, ApprovalAction::AdminConfirm));
    assert!(!viewer_has_acted(&r, ApprovalAction::ModeratorCosign));
}