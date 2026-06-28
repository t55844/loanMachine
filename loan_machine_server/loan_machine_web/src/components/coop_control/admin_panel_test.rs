use super::admin_panel::{action_button_label, viewer_has_acted, ApprovalAction, fmt_due_date, days_to_ymd};
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

// ── days_to_ymd ──────────────────────────────────────────────

#[test]
fn days_to_ymd_epoch_is_1970_01_01() {
    assert_eq!(days_to_ymd(0), (1970, 1, 1));
}

#[test]
fn days_to_ymd_one_day_is_1970_01_02() {
    assert_eq!(days_to_ymd(1), (1970, 1, 2));
}

#[test]
fn days_to_ymd_365_is_1971_01_01() {
    assert_eq!(days_to_ymd(365), (1971, 1, 1));
}

#[test]
fn days_to_ymd_leap_year_2000_02_29() {
    // 951 782 400 / 86 400 = 11 016 days since epoch
    assert_eq!(days_to_ymd(11_016), (2000, 2, 29));
}

#[test]
fn days_to_ymd_known_date_2024_01_01() {
    // 1 704 067 200 / 86 400 = 19 723 days since epoch
    assert_eq!(days_to_ymd(19_723), (2024, 1, 1));
}

#[test]
fn days_to_ymd_known_date_2023_12_31() {
    // 1 703 980 800 / 86 400 = 19 722 days since epoch
    assert_eq!(days_to_ymd(19_722), (2023, 12, 31));
}

// ── fmt_due_date ─────────────────────────────────────────────

#[test]
fn fmt_due_date_epoch_is_1970_01_01_midnight_utc() {
    assert_eq!(fmt_due_date(0), "1970-01-01 00:00:00 UTC");
}

#[test]
fn fmt_due_date_one_day_after_epoch() {
    assert_eq!(fmt_due_date(86_400), "1970-01-02 00:00:00 UTC");
}

#[test]
fn fmt_due_date_time_components_are_correct() {
    // 1 h + 1 min + 1 s = 3 661 s; date is still 1970-01-01
    assert_eq!(fmt_due_date(3_661), "1970-01-01 01:01:01 UTC");
}

#[test]
fn fmt_due_date_known_timestamp_2024_01_01() {
    // 2024-01-01 00:00:00 UTC = 1 704 067 200
    assert_eq!(fmt_due_date(1_704_067_200), "2024-01-01 00:00:00 UTC");
}

#[test]
fn fmt_due_date_leap_year_2000_02_29() {
    // 2000-02-29 00:00:00 UTC = 951 782 400
    assert_eq!(fmt_due_date(951_782_400), "2000-02-29 00:00:00 UTC");
}

#[test]
fn fmt_due_date_end_of_year_2023_12_31() {
    // 2023-12-31 00:00:00 UTC = 1 703 980 800
    assert_eq!(fmt_due_date(1_703_980_800), "2023-12-31 00:00:00 UTC");
}

#[test]
fn fmt_due_date_non_midnight_includes_time() {
    // 2024-01-01 13:45:59 UTC
    // = 1 704 067 200 + 13*3600 + 45*60 + 59 = 1 704 116 759
    assert_eq!(fmt_due_date(1_704_116_759), "2024-01-01 13:45:59 UTC");
}
