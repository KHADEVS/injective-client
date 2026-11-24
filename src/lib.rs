//! Build and sign Injective transactions.
//!
//! - [`key`]: Injective keys (`ethsecp256k1`), addresses and subaccount IDs.
//! - [`tx`]: transactions in `SIGN_MODE_DIRECT`.
//! - [`msg`]: bank transfers and exchange orders.
//! - [`dec`]: the wire format of Cosmos `LegacyDec` values (prices and quantities).
//!
//! The signing is checked byte for byte against Trust Wallet Core's Injective vector, and the
//! example in `examples/testnet.rs` sends real transactions to the Injective testnet.

pub mod dec;
pub mod key;
pub mod msg;
pub mod proto;
pub mod tx;

pub use key::{Address, PrivateKey, PubKeyEncoding, PublicKey};
pub use tx::{Fee, SignedTx, SignerData, TxBuilder};

/// Errors from this crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The bytes are not a valid secp256k1 private key.
    InvalidPrivateKey,
    /// The string is not a valid `inj` Bech32 address.
    InvalidAddress(String),
    /// The string is not a valid non-negative decimal with at most 18 fraction digits.
    InvalidDecimal(String),
    /// The string is not a `0x`-prefixed 32-byte market ID.
    InvalidMarketId(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidPrivateKey => write!(f, "invalid secp256k1 private key"),
            Error::InvalidAddress(a) => write!(f, "invalid Injective address: {a}"),
            Error::InvalidDecimal(d) => write!(f, "invalid decimal: {d}"),
            Error::InvalidMarketId(m) => write!(f, "invalid market ID: {m}"),
        }
    }
}

impl std::error::Error for Error {}
