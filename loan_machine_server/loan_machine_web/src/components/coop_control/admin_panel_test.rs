use super::admin_panel::{action_button_label, viewer_has_acted, ApprovalAction};
use loan_machine_models::responses::ApproveWalletProposalRow;

fn row(viewer_confirmed: bool) -> ApproveWalletProposalRow {
    ApproveWalletProposalRow {
        proposal_id:  0,
        proposer:     "0x0000000000000000000000000000000000000000".into(),
        created_at:   0,
        confirmations: 0,
        viewer_confirmed,
    }
}

#[test]
fn admin_button_idle_label() {
    assert_eq!(action_button_label(ApprovalAction::AdminConfirm, false), "Confirm");
}

#[test]
fn sending_label() {
    assert_eq!(action_button_label(ApprovalAction::AdminConfirm, true), "Sending…");
}

#[test]
fn admin_acted_iff_viewer_confirmed() {
    assert!( viewer_has_acted(&row(true),  ApprovalAction::AdminConfirm));
    assert!(!viewer_has_acted(&row(false), ApprovalAction::AdminConfirm));
}
