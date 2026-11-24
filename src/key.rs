//! Injective keys, addresses and subaccount IDs.
//!
//! Injective accounts use `ethsecp256k1` keys: secp256k1 keys whose address is the Ethereum
//! address (the last 20 bytes of the Keccak-256 hash of the uncompressed public key), encoded
//! in Bech32 with the `inj` prefix, and whose signatures are over the Keccak-256 hash of the
//! sign document.

use bech32::{primitives::decode::CheckedHrpstring, Bech32, Hrp};
use k256::ecdsa::{signature::hazmat::PrehashSigner, Signature, SigningKey, VerifyingKey};
use k256::elliptic_curve::rand_core::CryptoRngCore;
use sha3::{Digest, Keccak256};

use crate::Error;

/// The Bech32 prefix of Injective account addresses.
pub const HRP: &str = "inj";

/// How the public key is written in a transaction's signer info.
///
/// The official SDKs write the 33-byte compressed key; Trust Wallet Core's vector writes the
/// 65-byte uncompressed key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PubKeyEncoding {
    /// 33 bytes, `0x02` or `0x03` then X.
    Compressed,
    /// 65 bytes, `0x04` then X and Y.
    Uncompressed,
}

/// An Injective private key.
pub struct PrivateKey(SigningKey);

impl std::fmt::Debug for PrivateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PrivateKey(..)")
    }
}

impl PrivateKey {
    /// A private key from its 32 big-endian bytes.
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, Error> {
        SigningKey::from_bytes(bytes.into())
            .map(Self)
            .map_err(|_| Error::InvalidPrivateKey)
    }

    /// A new random private key.
    pub fn random(rng: &mut impl CryptoRngCore) -> Self {
        Self(SigningKey::random(rng))
    }

    /// The private key's 32 big-endian bytes.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes().into()
    }

    /// The public key.
    pub fn public_key(&self) -> PublicKey {
        PublicKey(*self.0.verifying_key())
    }

    /// Signs the Keccak-256 hash of `message`, as `ethsecp256k1` keys do.
    ///
    /// Deterministic (RFC 6979), with a low S, as 64 bytes `r || s`.
    pub fn sign_keccak256(&self, message: &[u8]) -> [u8; 64] {
        let digest = Keccak256::digest(message);
        let signature: Signature = self
            .0
            .sign_prehash(&digest)
            .expect("a 32-byte digest is a valid prehash");
        let signature = signature.normalize_s().unwrap_or(signature);
        signature.to_bytes().into()
    }
}

/// An Injective public key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PublicKey(VerifyingKey);

impl PublicKey {
    /// The public key in the given encoding.
    pub fn to_bytes(&self, encoding: PubKeyEncoding) -> Vec<u8> {
        self.0
            .to_encoded_point(encoding == PubKeyEncoding::Compressed)
            .as_bytes()
            .to_vec()
    }

    /// The account address.
    pub fn address(&self) -> Address {
        let point = self.0.to_encoded_point(false);
        let hash = Keccak256::digest(&point.as_bytes()[1..]);
        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(&hash[12..]);
        Address(bytes)
    }
}

/// An Injective account address: 20 bytes, shown as `inj1...`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Address([u8; 20]);

impl Address {
    /// An address from its 20 bytes (the account's Ethereum address).
    pub fn from_bytes(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    /// Parses an `inj1...` Bech32 address.
    pub fn from_bech32(s: &str) -> Result<Self, Error> {
        let invalid = || Error::InvalidAddress(s.to_string());
        let checked = CheckedHrpstring::new::<Bech32>(s).map_err(|_| invalid())?;
        if checked.hrp().as_str() != HRP {
            return Err(invalid());
        }
        let bytes: Vec<u8> = checked.byte_iter().collect();
        let bytes: [u8; 20] = bytes.try_into().map_err(|_| invalid())?;
        Ok(Self(bytes))
    }

    /// The address's 20 bytes.
    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }

    /// The ID of one of the account's exchange subaccounts: `0x`, the 20 address bytes, then
    /// the 12-byte nonce. Nonce 0 is the default subaccount, which trades from the bank balance.
    pub fn subaccount_id(&self, nonce: u32) -> String {
        let mut id = String::with_capacity(66);
        id.push_str("0x");
        for byte in self.0 {
            id.push_str(&format!("{byte:02x}"));
        }
        id.push_str(&format!("{nonce:024x}"));
        id
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hrp = Hrp::parse(HRP).expect("valid prefix");
        let encoded = bech32::encode::<Bech32>(hrp, &self.0).expect("20 bytes always encode");
        f.write_str(&encoded)
    }
}
