// Translates contract revert selectors into user-friendly messages.
//
// Every custom error in the Solidity contracts has a 4-byte selector equal to
// the first 4 bytes of keccak256("ErrorName()"). When a call reverts, alloy
// returns those bytes in the error payload. We match them here.

use alloy::primitives::Bytes;

/// Try to translate a revert payload into a friendly English message.
/// Returns None if the selector doesn't match a known contract error — the
/// caller should fall back to the raw alloy error in that case.
pub fn translate_revert(data: &[u8]) -> String {
    if data.len() < 4 {
        return "Unknown error (empty payload).".to_string();
    }
    let sel = [data[0], data[1], data[2], data[3]];
    match sel {
        // ── Reputation / Elections (RS_) ──────────────────────
        [0x1e, 0x7e, 0xc6, 0x03] => "Invalid member ID or wallet address.".into(),
        [0xf9, 0x44, 0x05, 0x51] => "This wallet is already linked.".into(),
        [0xee, 0x0c, 0xfd, 0xe8] => "This wallet is already associated with another member.".into(),
        [0x62, 0x97, 0xdd, 0x4e] => "An election is already active.".into(),
        [0x2e, 0x36, 0x7a, 0xef] => "No active election at this time.".into(),
        [0xde, 0x6b, 0x8f, 0xb6] => "This member has already voted in this election.".into(),
        [0x70, 0xdc, 0xb1, 0x5d] => "Invalid candidate.".into(),
        [0x18, 0x30, 0x4b, 0x1b] => "No candidates in the election.".into(),

        // ── Multisig admin ────────────────────────────────────
        [0xd8, 0x9d, 0x05, 0x20] => "Only administrators can perform this action.".into(),
        [0x86, 0x13, 0x08, 0x49] => "This address is already an administrator.".into(),
        [0xe1, 0xaa, 0xb7, 0x3b] => "Insufficient number of administrators.".into(),
        [0xe9, 0x46, 0xad, 0xc6] => "Invalid signature threshold.".into(),
        [0x32, 0x43, 0xbb, 0x79] => "This proposal has already been executed.".into(),
        [0x6a, 0x17, 0x07, 0x9f] => "You have already confirmed this proposal.".into(),
        [0x37, 0x1e, 0x2c, 0xc3] => "Proposal not found.".into(),

        // ── Wallet approval / moderator ───────────────────────
        [0xc6, 0xd4, 0x31, 0xc1] => "Wallet approval request not found.".into(),
        [0x31, 0xd7, 0xd4, 0x65] => "This wallet approval has already been executed.".into(),
        [0xf3, 0x9a, 0xa3, 0x32] => "Only the elected moderator can perform this action.".into(),
        [0x42, 0x42, 0x64, 0x21] => "Administrator has not been proposed yet.".into(),

        // ── Coop lifecycle / membership ───────────────────────
        [0x47, 0x6e, 0xbc, 0x32] => "Cooperative already initialized.".into(),
        [0x63, 0xfa, 0x89, 0x7b] => "Wallet not approved by the cooperative.".into(),
        [0x7f, 0x1a, 0x48, 0xc5] => "This wallet is already a member of the cooperative.".into(),
        [0x67, 0xf7, 0x6b, 0xa4] => "Invalid access code.".into(),
        [0x81, 0xe2, 0x31, 0xf6] => "This cooperative is not active.".into(),

        // ── Financial ops ─────────────────────────────────────
        [0x0d, 0x84, 0xc0, 0xf0] => "Invalid amount.".into(),
        [0xf8, 0xa1, 0x7f, 0xc4] => "Insufficient funds.".into(),
        [0x1b, 0xf0, 0xa1, 0x50] => "Loan term has not expired yet.".into(),
        [0x28, 0x19, 0x76, 0x0f] => "Invalid coverage percentage.".into(),
        [0x11, 0xd9, 0x35, 0x51] => "Coverage above the allowed limit.".into(),
        [0x72, 0x7c, 0xd2, 0x12] => "Loan not available.".into(),
        [0xe1, 0x93, 0x5a, 0x36] => "Maximum number of pending loans reached.".into(),
        [0x60, 0x44, 0xd4, 0xdd] => "Insufficient donation balance.".into(),
        [0xc4, 0x96, 0x7c, 0xf3] => "No active loan.".into(),
        [0x0a, 0x7d, 0x92, 0xaa] => "Invalid number of installments.".into(),
        [0x7a, 0x97, 0xd7, 0x1f] => "Token transfer failed.".into(),
        [0x58, 0x4d, 0xaf, 0x25] => "Invalid member ID or wallet address.".into(),
        [0x1c, 0xc5, 0x3e, 0x44] => "Minimum coverage percentage not reached.".into(),
        [0xdb, 0xee, 0x9d, 0x4f] => "Insufficient balance available for withdrawal.".into(),
        [0xc6, 0x71, 0xf0, 0x79] => "Only the borrower can cancel this request.".into(),
        [0xe0, 0x47, 0x4f, 0x8a] => "This request cannot be cancelled.".into(),
        [0x3e, 0xa1, 0xd7, 0x08] => "Request already fully covered.".into(),
        [0x36, 0x31, 0xf3, 0x1d] => "Payment interval above the limit.".into(),

        // ── Registry ──────────────────────────────────────────
        [0x68, 0x33, 0x15, 0x20] => "Only the platform admin can register cooperatives.".into(),
        [0x47, 0x2a, 0x45, 0x6d] => "Cooperative not found in the registry.".into(),
        [0xf0, 0x83, 0x02, 0x6c] => "Cooperative name cannot be empty.".into(),
        [0xc3, 0x7b, 0x9b, 0x52] => "This cooperative is already registered.".into(),

        _ => format!(
            "Unknown error (0x{:02x}{:02x}{:02x}{:02x}).",
            sel[0], sel[1], sel[2], sel[3]
        ),
    }
}

/// Extract the 4-byte selector payload from an alloy error, if any.
/// Works with the common shape of `TransportError(ErrorResp { data: ... })`.
pub fn extract_revert_data(err: &dyn std::error::Error) -> Option<Bytes> {
    // The payload is nested inside the error's Display form as hex.
    // For structured access, cast to alloy's specific error types.
    // Simplest approach that works across alloy versions: scan Debug output.
    let s = format!("{:?}", err);

    // Look for 0x-prefixed 8 hex chars (= 4 bytes = selector)
    let start = s.find("RawValue(\"0x")?;
    let rest  = &s[start + "RawValue(\"0x".len()..];
    let end   = rest.find('"')?;
    let hex   = &rest[..end];

    if hex.len() < 8 {
        return None;
    }
    hex::decode(hex).ok().map(Bytes::from)
}

/// Convenience: turn any blockchain error into a friendly string,
/// falling back to the underlying error's display if no selector matched.
pub fn friendly_from_error(err: &dyn std::error::Error) -> String {
    if let Some(bytes) = extract_revert_data(err) {
        return translate_revert(&bytes);
    }
    err.to_string()
}