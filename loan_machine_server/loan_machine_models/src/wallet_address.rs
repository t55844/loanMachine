// loan_machine_models/src/wallet_address.rs
use std::fmt;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use alloy::primitives::Address;


#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct WalletAddress([u8; 20]);

#[derive(Debug, thiserror::Error)]
pub enum WalletAddressParseError {
    #[error("address must start with 0x")]
    MissingPrefix,
    #[error("address must be 42 chars; got {0}")]
    BadLength(usize),
    #[error("invalid hex byte at position {0}")]
    BadHex(usize),
}

impl WalletAddress {
    pub fn as_bytes(&self) -> &[u8; 20]    { &self.0 }
    pub fn into_bytes(self) -> [u8; 20]    { self.0 }
    pub fn from_bytes(b: [u8; 20]) -> Self { Self(b) }

    /// Truncated display for UI: 0x1234…abcd.
    pub fn short(&self) -> String {
        let s = self.to_string();
        format!("{}…{}", &s[..6], &s[38..])
    }
}

impl fmt::Display for WalletAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("0x")?;
        for b in self.0 { write!(f, "{b:02x}")?; }
        Ok(())
    }
}

impl fmt::Debug for WalletAddress {
    // Same as Display so logs stay readable.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { fmt::Display::fmt(self, f) }
}

impl FromStr for WalletAddress {
    type Err = WalletAddressParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("0x") && !s.starts_with("0X") {
            return Err(WalletAddressParseError::MissingPrefix);
        }
        if s.len() != 42 {
            return Err(WalletAddressParseError::BadLength(s.len()));
        }
        let mut out = [0u8; 20];
        let hex = &s[2..];
        for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
            let hi = (chunk[0] as char).to_digit(16)
                .ok_or(WalletAddressParseError::BadHex(2 + i * 2))? as u8;
            let lo = (chunk[1] as char).to_digit(16)
                .ok_or(WalletAddressParseError::BadHex(2 + i * 2 + 1))? as u8;
            out[i] = (hi << 4) | lo;
        }
        Ok(Self(out))
    }
}

// Serde: wire form is a hex string, in-memory is bytes.
impl Serialize for WalletAddress {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.collect_str(self)  // uses Display
    }
}
impl<'de> Deserialize<'de> for WalletAddress {
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let s = String::deserialize(de)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}


pub fn to_alloy(w: &WalletAddress) -> Address { Address::from(*w.as_bytes()) }
pub fn from_alloy(a: Address) -> WalletAddress { WalletAddress::from_bytes(a.into_array()) }