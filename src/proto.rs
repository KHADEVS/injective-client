//! The protobuf messages this crate writes.
//!
//! Field numbers follow the Cosmos SDK and InjectiveLabs/injective-proto
//! (`injective/exchange/v2`, `injective/crypto/v1beta1/ethsecp256k1`).

/// Type URL of a bank transfer.
pub const MSG_SEND: &str = "/cosmos.bank.v1beta1.MsgSend";
/// Type URL of an Injective public key.
pub const ETH_SECP256K1_PUBKEY: &str = "/injective.crypto.v1beta1.ethsecp256k1.PubKey";
/// Type URL of a spot limit order (exchange module v2).
pub const MSG_CREATE_SPOT_LIMIT_ORDER: &str = "/injective.exchange.v2.MsgCreateSpotLimitOrder";

/// `SIGN_MODE_DIRECT`.
pub const SIGN_MODE_DIRECT: i32 = 1;

/// `google.protobuf.Any`.
#[derive(Clone, PartialEq, prost::Message)]
pub struct Any {
    #[prost(string, tag = "1")]
    pub type_url: String,
    #[prost(bytes = "vec", tag = "2")]
    pub value: Vec<u8>,
}

/// `cosmos.base.v1beta1.Coin`.
#[derive(Clone, PartialEq, prost::Message)]
pub struct Coin {
    #[prost(string, tag = "1")]
    pub denom: String,
    #[prost(string, tag = "2")]
    pub amount: String,
}

/// `cosmos.tx.v1beta1.TxBody` (without extension options).
#[derive(Clone, PartialEq, prost::Message)]
pub struct TxBody {
    #[prost(message, repeated, tag = "1")]
    pub messages: Vec<Any>,
    #[prost(string, tag = "2")]
    pub memo: String,
    #[prost(uint64, tag = "3")]
    pub timeout_height: u64,
}

/// `cosmos.tx.v1beta1.ModeInfo.Single`.
#[derive(Clone, PartialEq, prost::Message)]
pub struct ModeInfoSingle {
    #[prost(int32, tag = "1")]
    pub mode: i32,
}

/// `cosmos.tx.v1beta1.ModeInfo` (single signer only).
#[derive(Clone, PartialEq, prost::Message)]
pub struct ModeInfo {
    #[prost(message, optional, tag = "1")]
    pub single: Option<ModeInfoSingle>,
}

/// `cosmos.tx.v1beta1.SignerInfo`.
#[derive(Clone, PartialEq, prost::Message)]
pub struct SignerInfo {
    #[prost(message, optional, tag = "1")]
    pub public_key: Option<Any>,
    #[prost(message, optional, tag = "2")]
    pub mode_info: Option<ModeInfo>,
    #[prost(uint64, tag = "3")]
    pub sequence: u64,
}

/// `cosmos.tx.v1beta1.Fee`.
#[derive(Clone, PartialEq, prost::Message)]
pub struct Fee {
    #[prost(message, repeated, tag = "1")]
    pub amount: Vec<Coin>,
    #[prost(uint64, tag = "2")]
    pub gas_limit: u64,
    #[prost(string, tag = "3")]
    pub payer: String,
    #[prost(string, tag = "4")]
    pub granter: String,
}

/// `cosmos.tx.v1beta1.AuthInfo`.
#[derive(Clone, PartialEq, prost::Message)]
pub struct AuthInfo {
    #[prost(message, repeated, tag = "1")]
    pub signer_infos: Vec<SignerInfo>,
    #[prost(message, optional, tag = "2")]
    pub fee: Option<Fee>,
}

/// `cosmos.tx.v1beta1.SignDoc`.
#[derive(Clone, PartialEq, prost::Message)]
pub struct SignDoc {
    #[prost(bytes = "vec", tag = "1")]
    pub body_bytes: Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub auth_info_bytes: Vec<u8>,
    #[prost(string, tag = "3")]
    pub chain_id: String,
    #[prost(uint64, tag = "4")]
    pub account_number: u64,
}

/// `cosmos.tx.v1beta1.TxRaw`.
#[derive(Clone, PartialEq, prost::Message)]
pub struct TxRaw {
    #[prost(bytes = "vec", tag = "1")]
    pub body_bytes: Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub auth_info_bytes: Vec<u8>,
    #[prost(bytes = "vec", repeated, tag = "3")]
    pub signatures: Vec<Vec<u8>>,
}

/// `cosmos.bank.v1beta1.MsgSend`.
#[derive(Clone, PartialEq, prost::Message)]
pub struct MsgSend {
    #[prost(string, tag = "1")]
    pub from_address: String,
    #[prost(string, tag = "2")]
    pub to_address: String,
    #[prost(message, repeated, tag = "3")]
    pub amount: Vec<Coin>,
}

/// `injective.crypto.v1beta1.ethsecp256k1.PubKey`.
#[derive(Clone, PartialEq, prost::Message)]
pub struct EthSecp256k1PubKey {
    #[prost(bytes = "vec", tag = "1")]
    pub key: Vec<u8>,
}

/// `injective.exchange.v2.OrderInfo`. `price` and `quantity` are `LegacyDec` in wire form.
#[derive(Clone, PartialEq, prost::Message)]
pub struct OrderInfo {
    #[prost(string, tag = "1")]
    pub subaccount_id: String,
    #[prost(string, tag = "2")]
    pub fee_recipient: String,
    #[prost(string, tag = "3")]
    pub price: String,
    #[prost(string, tag = "4")]
    pub quantity: String,
    #[prost(string, tag = "5")]
    pub cid: String,
}

/// `injective.exchange.v2.SpotOrder`.
#[derive(Clone, PartialEq, prost::Message)]
pub struct SpotOrder {
    #[prost(string, tag = "1")]
    pub market_id: String,
    #[prost(message, optional, tag = "2")]
    pub order_info: Option<OrderInfo>,
    #[prost(int32, tag = "3")]
    pub order_type: i32,
    /// `LegacyDec` in wire form; empty for a limit order.
    #[prost(string, tag = "4")]
    pub trigger_price: String,
    #[prost(int64, tag = "5")]
    pub expiration_block: i64,
}

/// `injective.exchange.v2.MsgCreateSpotLimitOrder`.
#[derive(Clone, PartialEq, prost::Message)]
pub struct MsgCreateSpotLimitOrder {
    #[prost(string, tag = "1")]
    pub sender: String,
    #[prost(message, optional, tag = "2")]
    pub order: Option<SpotOrder>,
}
