
use loan_machine_core::services::blockchain::contract_errors::{translate_revert};

    // ── helper ────────────────────────────────────────────────
    // Build a minimal revert payload: 4-byte selector + optional ABI padding.
    fn payload(a: u8, b: u8, c: u8, d: u8) -> Vec<u8> {
        vec![a, b, c, d]
    }

    // ── STEP: prepare_deploy_bundle (TX1 calldata building) ───
    // These revert inside initializeMultisig, which the founder signs.

    #[test]
    fn already_initialized() {
        // initializeMultisig called twice
        assert_eq!(
            translate_revert(&payload(0x47, 0x6e, 0xbc, 0x32)),
            "Cooperative already initialized."
        );
    }

    #[test]
    fn invalid_threshold() {
        // threshold = 0 or > admin count
        assert_eq!(
            translate_revert(&payload(0xe9, 0x46, 0xad, 0xc6)),
            "Invalid signature threshold."
        );
    }

    #[test]
    fn only_admins_can_act() {
        // a non-admin wallet tries to call an admin-only function
        assert_eq!(
            translate_revert(&payload(0xd8, 0x9d, 0x05, 0x20)),
            "Only administrators can perform this action."
        );
    }

    // ── STEP: register_deployed_coop (TX2, platform admin signs) ─

    #[test]
    fn only_platform_admin_can_register() {
        // registerCoop called from a wallet that isn't the platform admin
        assert_eq!(
            translate_revert(&payload(0x68, 0x33, 0x15, 0x20)),
            "Only the platform admin can register cooperatives."
        );
    }

    #[test]
    fn coop_already_registered() {
        // same LoanMachine address registered twice
        assert_eq!(
            translate_revert(&payload(0xc3, 0x7b, 0x9b, 0x52)),
            "This cooperative is already registered."
        );
    }

    #[test]
    fn coop_name_empty() {
        // registerCoop called with empty name string
        assert_eq!(
            translate_revert(&payload(0xf0, 0x83, 0x02, 0x6c)),
            "Cooperative name cannot be empty."
        );
    }

    #[test]
    fn coop_not_found_in_registry() {
        // querying a coopId that was never registered
        assert_eq!(
            translate_revert(&payload(0x47, 0x2a, 0x45, 0x6d)),
            "Cooperative not found in the registry."
        );
    }

    // ── STEP: membership after registration ───────────────────

    #[test]
    fn wallet_not_approved() {
        // member tries to join without approval
        assert_eq!(
            translate_revert(&payload(0x63, 0xfa, 0x89, 0x7b)),
            "Wallet not approved by the cooperative."
        );
    }

    #[test]
    fn wallet_already_member() {
        // joinCoop called by someone already in the coop
        assert_eq!(
            translate_revert(&payload(0x7f, 0x1a, 0x48, 0xc5)),
            "This wallet is already a member of the cooperative."
        );
    }

    #[test]
    fn coop_not_active() {
        // action attempted on a deactivated coop
        assert_eq!(
            translate_revert(&payload(0x81, 0xe2, 0x31, 0xf6)),
            "This cooperative is not active."
        );
    }

    // ── Edge cases ────────────────────────────────────────────

    #[test]
    fn payload_too_short_returns_empty_message() {
        let msg = translate_revert(&[0x01, 0x02]);
        assert_eq!(msg, "Unknown error (empty payload).");
    }

    #[test]
    fn unknown_selector_includes_hex() {
        // An unrecognised selector should come back with its hex so
        // developers can identify and add it to the table later.
        let msg = translate_revert(&payload(0xde, 0xad, 0xbe, 0xef));
        assert!(
            msg.contains("0xdeadbeef"),
            "expected hex in fallback message, got: {msg}"
        );
    }

    #[test]
    fn extra_bytes_after_selector_are_ignored() {
        // ABI-encoded errors have 32+ bytes of padding after the selector.
        // We only look at the first 4.
        let mut data = payload(0xc3, 0x7b, 0x9b, 0x52);
        data.extend_from_slice(&[0u8; 32]); // ABI padding
        assert_eq!(translate_revert(&data), "This cooperative is already registered.");
    }
