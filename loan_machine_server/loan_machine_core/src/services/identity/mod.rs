// src/services/identity/mod.rs
//
// Responsibilities:
//   1. Load / validate the COOP_SALT env var (32-byte hex)
//   2. Compute memberId = keccak256(salt ++ wallet_address_bytes)
//      → the bytes32 that goes on-chain / into the subgraph
//   3. NEVER persist raw wallet bytes anywhere — only the hash leaves this module

use alloy::primitives::{keccak256, FixedBytes};

// ── ERRORS ───────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum IdentityError {
    #[error("COOP_SALT not configured or invalid (expected: 64 hex chars = 32 bytes)")]
    SaltMissing,
}

pub struct IdentityService {
    salt: [u8; 32],
}

impl IdentityService {
    pub fn from_env() -> Result<Self, IdentityError> {
        let salt = load_salt_from_env()?;
        Ok(Self { salt })
    }

    /// Derive a deterministic, privacy-preserving member ID from a wallet address.
    /// `member_id = keccak256(COOP_SALT ++ wallet_bytes)`
    pub fn wallet_to_member_id(&self, wallet_bytes: &[u8; 20]) -> FixedBytes<32> {
        let mut input = [0u8; 52];
        input[..32].copy_from_slice(&self.salt);
        input[32..].copy_from_slice(wallet_bytes);
        keccak256(&input)
    }

    #[cfg(any(test, feature = "deployable"))]
    pub fn with_salt(salt: [u8; 32]) -> Self {
        Self { salt }
    }
}

impl std::fmt::Debug for IdentityService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("IdentityService ([REDACTED])")
    }
}

// ── SALT UTILITIES ───────────────────────────────────────────

pub fn load_salt_from_env() -> Result<[u8; 32], IdentityError> {
    let hex = std::env::var("COOP_SALT").map_err(|_| IdentityError::SaltMissing)?;
    let bytes = hex::decode(hex.trim()).map_err(|_| IdentityError::SaltMissing)?;
    if bytes.len() != 32 {
        return Err(IdentityError::SaltMissing);
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}

pub fn generate_and_print_salt() -> [u8; 32] {
    use rand::RngCore;
    let mut salt = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt);
    println!("Generated COOP_SALT (hex): {}", hex::encode(salt));
    salt
}

// ── TESTS ─────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const WALLET_A: &[u8; 20] = b"\x11\x11\x11\x11\x11\x11\x11\x11\x11\x11\x11\x11\x11\x11\x11\x11\x11\x11\x11\x11";
    const WALLET_B: &[u8; 20] = b"\x22\x22\x22\x22\x22\x22\x22\x22\x22\x22\x22\x22\x22\x22\x22\x22\x22\x22\x22\x22";

    #[test]
    fn same_wallet_same_salt_produces_same_id() {
        let svc = IdentityService::with_salt([0xABu8; 32]);
        let id1 = svc.wallet_to_member_id(WALLET_A);
        let id2 = svc.wallet_to_member_id(WALLET_A);
        assert_eq!(id1, id2);
    }

    #[test]
    fn different_wallets_same_salt_produce_different_ids() {
        let svc = IdentityService::with_salt([0xABu8; 32]);
        let id_a = svc.wallet_to_member_id(WALLET_A);
        let id_b = svc.wallet_to_member_id(WALLET_B);
        assert_ne!(id_a, id_b);
    }

    #[test]
    fn same_wallet_different_salts_produce_different_ids() {
        let svc1 = IdentityService::with_salt([0x01u8; 32]);
        let svc2 = IdentityService::with_salt([0x02u8; 32]);
        let id1 = svc1.wallet_to_member_id(WALLET_A);
        let id2 = svc2.wallet_to_member_id(WALLET_A);
        assert_ne!(id1, id2);
    }

    #[test]
    fn output_is_32_bytes() {
        let svc = IdentityService::with_salt([0x00u8; 32]);
        let id = svc.wallet_to_member_id(WALLET_A);
        assert_eq!(id.len(), 32);
    }
}
