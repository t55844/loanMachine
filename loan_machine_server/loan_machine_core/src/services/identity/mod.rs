// src/services/identity/mod.rs
//
// Phase A — Server-Side Identity
//
// Responsibilities:
//   1. Load / validate the COOP_SALT env var (32-byte hex)
//   2. Validate Brazilian CPF (individuals) and CNPJ (companies) — mod-11
//   3. Compute memberId = keccak256(salt ++ cpf_digits_as_bytes)
//      → the bytes32 that goes on-chain / into the subgraph
//   4. NEVER persist raw CPF/CNPJ anywhere — only the hash leaves this module

use alloy::primitives::{keccak256, FixedBytes};

// ── ERRORS ───────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum IdentityError{
    #[error("CPF must have 11 digits")]
    CpfLength,
    #[error("Invalid CPF: incorrect check digits")]
    CpfCheckDigits,
    #[error("Invalid CPF: all digits are the same")]
    CpfAllSame,
    #[error("CNPJ must have 14 digits")]
    CnpjLength,
    #[error("Invalid CNPJ: incorrect check digits")]
    CnpjCheckDigits,
    #[error("Invalid CNPJ: all digits are the same")]
    CnpjAllSame,
    #[error("COOP_SALT not configured or invalid (expected: 64 hex chars = 32 bytes)")]
    SaltMissing,
}

pub struct IdentityService{
    salt: [u8; 32],
}

impl IdentityService{
    pub fn  from_env() -> Result<Self, IdentityError>{
        let salt = load_salt_from_env()?;
        Ok(Self{salt})
    }

    pub fn cpf_to_member_id(&self, raw_cpf: &str) -> Result<FixedBytes<32>, IdentityError>{
        let digits = validate_cpf(raw_cpf)?;
        let id = hash_member_id(&self.salt, &digits);
        drop(digits);
        Ok(id)
    }

    pub fn cnpj_to_member_id(&self, raw_cnpj: &str) -> Result<FixedBytes<32>, IdentityError>{
        let digits = validate_cnpj(raw_cnpj)?;
        let id = hash_member_id(&self.salt, &digits);
        drop(digits);
        Ok(id)
    }

    #[cfg(any(test, feature = "deployable"))]
    pub fn with_salt(salt: [u8; 32]) -> Self {
        Self { salt }
    }
}

impl std::fmt::Debug for IdentityService{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result{
        f.write_str("IdentityService ([REDACTED])")
    }
}

// ── CPF VALIDATION ───────────────────────────────────────────
//
// Algorithm (Brazilian Receita Federal):
//   weights₁ = [10, 9, 8, 7, 6, 5, 4, 3, 2]    → check digit d[9]
//   weights₂ = [11, 10, 9, 8, 7, 6, 5, 4, 3, 2] → check digit d[10]
//   remainder = sum % 11
//   digit     = 0 if remainder < 2, else 11 - remainder

/// Strip non-digits from a raw CPF string ("123.456.789-09" → "12345678909")
fn strip_non_digits(raw: &str) -> String {
    raw.chars().filter(|c| c.is_ascii_digit()).collect()
}

/// Validate a CPF string (formatted or bare digits).
/// Returns the 11 bare digits on success — drop this string after use.
pub fn validate_cpf(raw: &str) -> Result<String, IdentityError>{
    let digits = strip_non_digits(raw);

    if digits.len() != 11{
        return Err(IdentityError::CpfLength);
    }

    if digits.chars().all(|c| c == digits.chars().next().unwrap()){
        return Err(IdentityError::CpfAllSame);
    }

    let d: Vec<u32> = digits.chars().map(|c| c.to_digit(10).unwrap()).collect();

    let sum1: u32 = (0..9).map(|i| d[i] * (10 - i as u32)).sum();
    let rem1 = sum1 % 11;
    let check1 = if rem1 < 2{0} else {11 - rem1};
    if d[9] != check1{
        return Err(IdentityError::CpfCheckDigits);
    }

    let sum2: u32 = (0..10).map(|i| d[i] * (11 - i as u32)).sum();
    let rem2 = sum2 % 11;
    let check2 = if rem2 < 2{0} else {11 - rem2};
    if d[10] != check2{
        return Err(IdentityError::CpfCheckDigits);
    }

    Ok(digits)
}


pub fn validate_cnpj(raw: &str) -> Result<String, IdentityError> {
    // Strip separators and uppercase — base chars can be A-Z or 0-9 (RF 2026 format).
    let chars = strip_cnpj_formatting(raw);

    if chars.len() != 14 {
        return Err(IdentityError::CnpjLength);
    }

    if chars.chars().all(|c| c == chars.chars().next().unwrap()) {
        return Err(IdentityError::CnpjAllSame);
    }

    // Check digits (positions 13-14) are always numeric.
    if !chars[12..].chars().all(|c| c.is_ascii_digit()) {
        return Err(IdentityError::CnpjCheckDigits);
    }

    let vals: Vec<u32> = chars.chars().map(cnpj_char_val).collect();

    const W1: [u32; 12] = [5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
    let sum1: u32 = (0..12).map(|i| vals[i] * W1[i]).sum();
    let rem1 = sum1 % 11;
    let check1 = if rem1 < 2 { 0 } else { 11 - rem1 };
    if vals[12] != check1 {
        return Err(IdentityError::CnpjCheckDigits);
    }

    const W2: [u32; 13] = [6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
    let sum2: u32 = (0..13).map(|i| vals[i] * W2[i]).sum();
    let rem2 = sum2 % 11;
    let check2 = if rem2 < 2 { 0 } else { 11 - rem2 };
    if vals[13] != check2 {
        return Err(IdentityError::CnpjCheckDigits);
    }

    Ok(chars)
}

/// Strip CNPJ visual separators (`.`, `/`, `-`) and uppercase the result.
/// Only alphanumeric ASCII characters survive — anything else is removed.
fn strip_cnpj_formatting(raw: &str) -> String {
    raw.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

/// Map a CNPJ character to its numeric weight value.
/// Per SERPRO/RF spec: value = ASCII decimal - 48.
/// Digits 0-9 → 0-9. Letters A-Z → 17-42 ('A'=65-48=17, 'Z'=90-48=42).
fn cnpj_char_val(c: char) -> u32 {
    c as u32 - 48
}

// ── HASHING ──────────────────────────────────────────────────
//
// memberId = keccak256(abi.encodePacked(salt, cpf_bytes))
//
// This matches the Solidity side:
//   keccak256(abi.encodePacked(COOP_SALT, cpfBytes))
//
// Raw CPF digits are dropped by the caller immediately after this call.

/// Compute the on-chain memberId from the salt and validated CPF digits.
/// `cpf_digits` must come from `validate_cpf()` — bare digits only.
///
/// The returned FixedBytes<32> is safe to log / store; it reveals nothing
/// about the original CPF without the salt.
pub fn hash_member_id(salt: &[u8;32], cpf_digits: &str) -> FixedBytes<32>{
    let mut input = Vec::with_capacity(32 + cpf_digits.len());
    input.extend_from_slice(salt);
    input.extend_from_slice(cpf_digits.as_bytes());
    keccak256(&input)
}

pub fn load_salt_from_env() -> Result<[u8; 32], IdentityError>{
    let hex = std::env::var("COOP_SALT").map_err(|_| IdentityError::SaltMissing)?;
    let bytes = hex::decode(hex.trim()).map_err(|_| IdentityError::SaltMissing)?;
    if bytes.len() != 32 {
        return Err(IdentityError::SaltMissing);
    }

    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}

pub fn generate_and_print_salt() -> [u8; 32]{
    use rand::RngCore;
    let mut salt = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt);
    println!("Generated COOP_SALT (hex): {}", hex::encode(salt));
    salt
}

pub fn cpf_to_member_id(
    salt: &[u8; 32],
    raw_cpf: &str,
) -> Result<FixedBytes<32>, IdentityError>{
    let digits = validate_cpf(raw_cpf)?;
    let id = hash_member_id(salt, &digits);
    drop(digits); // ensure raw CPF digits are not used after this point
    Ok(id)
}

// ── TESTS ─────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Known-good CPFs generated by the standard algorithm
    const VALID_CPF_FORMATTED: &str = "529.982.247-25";
    const VALID_CPF_BARE: &str      = "52998224725";
    const INVALID_CPF_DIGITS: &str  = "52998224726";  // last digit off
    const ALL_SAME_CPF: &str        = "111.111.111-11";

    #[test]
    fn cpf_formatted_passes() {
        assert!(validate_cpf(VALID_CPF_FORMATTED).is_ok());
    }

    #[test]
    fn cpf_bare_passes() {
        assert!(validate_cpf(VALID_CPF_BARE).is_ok());
    }

    #[test]
    fn cpf_strips_formatting() {
        let result = validate_cpf(VALID_CPF_FORMATTED).unwrap();
        assert_eq!(result, VALID_CPF_BARE);
    }

    #[test]
    fn cpf_bad_check_digit_fails() {
        assert!(matches!(
            validate_cpf(INVALID_CPF_DIGITS),
            Err(IdentityError::CpfCheckDigits)
        ));
    }

    #[test]
    fn cpf_all_same_fails() {
        assert!(matches!(
            validate_cpf(ALL_SAME_CPF),
            Err(IdentityError::CpfAllSame)
        ));
    }

    #[test]
    fn cpf_too_short_fails() {
        assert!(matches!(
            validate_cpf("1234"),
            Err(IdentityError::CpfLength)
        ));
    }

    #[test]
    fn same_cpf_same_salt_produces_same_hash() {
        let salt = [0xABu8; 32];
        let h1   = cpf_to_member_id(&salt, VALID_CPF_FORMATTED).unwrap();
        let h2   = cpf_to_member_id(&salt, VALID_CPF_BARE).unwrap();
        assert_eq!(h1, h2, "formatted and bare CPF must hash identically");
    }

    #[test]
    fn different_salts_produce_different_hashes() {
        let salt1 = [0x01u8; 32];
        let salt2 = [0x02u8; 32];
        let h1    = cpf_to_member_id(&salt1, VALID_CPF_BARE).unwrap();
        let h2    = cpf_to_member_id(&salt2, VALID_CPF_BARE).unwrap();
        assert_ne!(h1, h2);
    }

    #[test]
    fn different_cpfs_same_salt_produce_different_hashes() {
        let salt = [0xABu8; 32];
        // Second valid CPF for comparison
        let h1 = cpf_to_member_id(&salt, VALID_CPF_BARE).unwrap();
        let h2 = cpf_to_member_id(&salt, "123.456.789-09").unwrap(); // another valid CPF
        assert_ne!(h1, h2);
    }

    // CNPJ — 11.222.333/0001-81 is a well-known test CNPJ (pure-numeric, old format)
    #[test]
    fn cnpj_formatted_passes() {
        assert!(validate_cnpj("11.222.333/0001-81").is_ok());
    }

    #[test]
    fn cnpj_bare_digits_passes() {
        assert!(validate_cnpj("11222333000181").is_ok());
    }

    #[test]
    fn cnpj_bad_check_digit_fails() {
        assert!(matches!(
            validate_cnpj("11.222.333/0001-82"),
            Err(IdentityError::CnpjCheckDigits)
        ));
    }

    // CNPJ — RF 2026 alphanumeric format; check digits for 12.ABC.345/0001 are 88
    // (per SERPRO spec: letter value = ASCII - 48, so A=17, B=18, C=19, …)
    #[test]
    fn cnpj_alphanumeric_formatted_passes() {
        assert!(validate_cnpj("12.ABC.345/0001-88").is_ok(),
            "RF 2026 alphanumeric CNPJ must be accepted");
    }

    #[test]
    fn cnpj_alphanumeric_bare_passes() {
        assert!(validate_cnpj("12ABC345000188").is_ok());
    }

    #[test]
    fn cnpj_alphanumeric_strips_formatting() {
        let result = validate_cnpj("12.ABC.345/0001-88").unwrap();
        assert_eq!(result, "12ABC345000188");
    }

    #[test]
    fn cnpj_alphanumeric_bad_check_digit_fails() {
        assert!(matches!(
            validate_cnpj("12.ABC.345/0001-89"),
            Err(IdentityError::CnpjCheckDigits)
        ));
    }

    #[test]
    fn cnpj_non_digit_check_positions_fail() {
        // Check digit positions (13-14) must always be numeric per the spec.
        assert!(matches!(
            validate_cnpj("12.ABC.345/0001-VV"),
            Err(IdentityError::CnpjCheckDigits)
        ));
    }
}