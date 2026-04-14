// tests/common/hashError.rs

use alloy::primitives::keccak256;

/// Scans a string for a 0x???????? selector and decodes it.
pub fn extract_and_decode(text: &str) -> String {
    // find all "0x" occurrences and try to pull 8+ hex chars after each
    let mut pos = 0;
    while let Some(start) = text[pos..].find("0x") {
        let abs_start = pos + start;
        let hex_part = &text[abs_start + 2..]; // skip "0x"
        let hex_len = hex_part
            .chars()
            .take_while(|c| c.is_ascii_hexdigit())
            .count();

        // a selector is exactly 8 hex chars (4 bytes)
        if hex_len >= 8 {
            let selector = &text[abs_start..abs_start + 2 + 8]; // "0x" + 8 chars
            let decoded = decode_revert_hash_error(selector);
            if !decoded.starts_with("unknown") {
                return decoded;
            }
        }
        pos = abs_start + 2;
    }
    "no known selector found in response".to_string()
}

/// Maps a 4-byte Solidity custom-error selector to a human-readable name.
pub fn decode_revert_hash_error(hex_data: &str) -> String {
    let hex = hex_data.strip_prefix("0x").unwrap_or(hex_data);
    if hex.len() < 8 {
        return format!("unknown revert data: 0x{hex}");
    }

    let selector = &hex[..8];

    const ERRORS: &[&str] = &[
        // CoopAccount
        "CoopAccount_NotOwner()",
        "CoopAccount_NotGuardian()",
        "CoopAccount_OnlyLoanMachine()",
        "CoopAccount_AlreadyApproved()",
        "CoopAccount_AlreadyInitialized()",
        "CoopAccount_CallFailed()",
        // RepresentativeSystem
        "RS_MemberIdOrWalletInvalid()",
        "RS_WalletAlreadyVinculated()",
        "RS_WalletAlreadyLinkedToAnotherMember()",
        "RS_ActiveElectionExists()",
        "RS_ElectionNotActive()",
        "RS_MemberAlreadyVoted()",
        "RS_InvalidCandidate()",
        "RS_NoCandidates()",
        // Factory
        "Factory_NotPlatformAdmin()",
        "Factory_CoopNotFound()",
        "Factory_NameEmpty()",
        // LoanMachine
        "LoanMachine_NotCoopAdmin()",
        "LoanMachine_AlreadyInitialized()",
        "LoanMachine_NotApproved()",
        "LoanMachine_AlreadyMember()",
        "LoanMachine_InvalidAccessCode()",
        "LoanMachine_CoopNotActive()",
        "LoanMachine_InvalidAmount()",
        "LoanMachine_InsufficientFunds()",
        "LoanMachine_MinimumDonationRequired()",
        "LoanMachine_BorrowNotExpired()",
        "LoanMachine_InvalidCoveragePercentage()",
        "LoanMachine_OverCoverage()",
        "LoanMachine_LoanNotAvailable()",
        "LoanMachine_MaxLoanRequisitionPendingReached()",
        "LoanMachine_InsufficientDonationBalance()",
        "LoanMachine_NoActiveBorrowing()",
        "LoanMachine_InvalidParcelsCount()",
        "LoanMachine_ExcessiveDonation()",
        "LoanMachine_TokenTransferFailed()",
        "LoanMachine_MemberIdOrWalletInvalid()",
        "LoanMachine_WalletAlreadyVinculated()",
        "LoanMachine_MinimumPercentageCover()",
        "LoanMachine_InsufficientWithdrawableBalance()",
        "LoanMachine_CheckIntervalNotYetPassed()",
        "LoanMachine_OnlyBorrowerCanCancelRequisition()",
        "LoanMachine_RequisitionNotCancellable()",
        "LoanMachine_RequisitionAlreadyFullyCovered()",
        "LoanMachine_IntervalOfPaymentAboveLimit()",
    ];

    for sig in ERRORS {
        let hash = keccak256(sig.as_bytes());
        let candidate = hex::encode(&hash[..4]);
        if candidate == selector {
            // strip the "()" for readability
            let name = sig.strip_suffix("()").unwrap_or(sig);
            return name.to_string();
        }
    }

    format!("unknown selector: 0x{selector}")
}