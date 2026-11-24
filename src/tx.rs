//! Transactions in `SIGN_MODE_DIRECT`.

use prost::Message;
use sha2::{Digest, Sha256};

use crate::proto::{self, Any, Coin};
use crate::{PrivateKey, PubKeyEncoding};

/// A transaction fee.
#[derive(Clone, Debug, PartialEq)]
pub struct Fee {
    /// The amount paid.
    pub amount: Vec<Coin>,
    /// The gas limit.
    pub gas_limit: u64,
}

/// The messages and memo of a transaction, before signing.
#[derive(Clone, Debug, Default)]
pub struct TxBuilder {
    messages: Vec<Any>,
    memo: String,
    timeout_height: u64,
}

/// What a signature commits to besides the transaction: the chain and the signer's account.
#[derive(Clone, Debug)]
pub struct SignerData<'a> {
    /// The chain ID, `injective-1` on Mainnet and `injective-888` on testnet.
    pub chain_id: &'a str,
    /// The signer's account number.
    pub account_number: u64,
    /// The signer's sequence.
    pub sequence: u64,
}

/// A signed transaction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignedTx {
    /// The `TxRaw` bytes to broadcast.
    pub tx_bytes: Vec<u8>,
}

impl SignedTx {
    /// The transaction hash, as explorers show it: SHA-256 of the bytes, uppercase hex.
    pub fn hash(&self) -> String {
        Sha256::digest(&self.tx_bytes)
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect()
    }
}

impl TxBuilder {
    /// An empty transaction.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a message.
    pub fn message(mut self, message: Any) -> Self {
        self.messages.push(message);
        self
    }

    /// Sets the memo.
    pub fn memo(mut self, memo: &str) -> Self {
        self.memo = memo.to_string();
        self
    }

    /// Sets the block height after which the transaction is invalid (0: none).
    pub fn timeout_height(mut self, height: u64) -> Self {
        self.timeout_height = height;
        self
    }

    /// Signs the transaction in `SIGN_MODE_DIRECT` with one key.
    pub fn sign_direct(
        &self,
        key: &PrivateKey,
        encoding: PubKeyEncoding,
        signer: &SignerData,
        fee: &Fee,
    ) -> SignedTx {
        let body_bytes = proto::TxBody {
            messages: self.messages.clone(),
            memo: self.memo.clone(),
            timeout_height: self.timeout_height,
        }
        .encode_to_vec();

        let public_key = Any {
            type_url: proto::ETH_SECP256K1_PUBKEY.to_string(),
            value: proto::EthSecp256k1PubKey {
                key: key.public_key().to_bytes(encoding),
            }
            .encode_to_vec(),
        };
        let auth_info_bytes = proto::AuthInfo {
            signer_infos: vec![proto::SignerInfo {
                public_key: Some(public_key),
                mode_info: Some(proto::ModeInfo {
                    single: Some(proto::ModeInfoSingle {
                        mode: proto::SIGN_MODE_DIRECT,
                    }),
                }),
                sequence: signer.sequence,
            }],
            fee: Some(proto::Fee {
                amount: fee.amount.clone(),
                gas_limit: fee.gas_limit,
                payer: String::new(),
                granter: String::new(),
            }),
        }
        .encode_to_vec();

        let sign_doc = proto::SignDoc {
            body_bytes: body_bytes.clone(),
            auth_info_bytes: auth_info_bytes.clone(),
            chain_id: signer.chain_id.to_string(),
            account_number: signer.account_number,
        }
        .encode_to_vec();
        let signature = key.sign_keccak256(&sign_doc);

        SignedTx {
            tx_bytes: proto::TxRaw {
                body_bytes,
                auth_info_bytes,
                signatures: vec![signature.to_vec()],
            }
            .encode_to_vec(),
        }
    }
}
