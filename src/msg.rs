//! Messages: bank transfers and exchange orders.

use prost::Message;

use crate::proto::{self, Any, Coin};
use crate::{dec, Address, Error};

/// A coin amount: a denomination and an integer amount in its base unit.
pub fn coin(denom: &str, amount: &str) -> Coin {
    Coin {
        denom: denom.to_string(),
        amount: amount.to_string(),
    }
}

/// A bank transfer.
pub fn bank_send(from: &Address, to: &Address, amount: Vec<Coin>) -> Any {
    let msg = proto::MsgSend {
        from_address: from.to_string(),
        to_address: to.to_string(),
        amount,
    };
    Any {
        type_url: proto::MSG_SEND.to_string(),
        value: msg.encode_to_vec(),
    }
}

/// The side of a limit order (`injective.exchange.v2.OrderType`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// `BUY`.
    Buy,
    /// `SELL`.
    Sell,
    /// `BUY_PO`: a buy that is only placed if it rests on the book.
    BuyPostOnly,
    /// `SELL_PO`: a sell that is only placed if it rests on the book.
    SellPostOnly,
}

impl Side {
    fn order_type(self) -> i32 {
        match self {
            Side::Buy => 1,
            Side::Sell => 2,
            Side::BuyPostOnly => 7,
            Side::SellPostOnly => 8,
        }
    }
}

/// A spot limit order. Price and quantity are human-readable decimals (exchange module v2),
/// for example a price of `12.5` quote units per base unit.
#[derive(Clone, Debug)]
pub struct SpotLimitOrder<'a> {
    /// The market ID, `0x` and 64 hex digits.
    pub market_id: &'a str,
    /// Buy or sell.
    pub side: Side,
    /// The limit price.
    pub price: &'a str,
    /// The quantity in base units.
    pub quantity: &'a str,
    /// The subaccount nonce; 0 trades from the bank balance.
    pub subaccount_nonce: u32,
    /// An optional client order ID.
    pub cid: Option<&'a str>,
}

/// A `MsgCreateSpotLimitOrder` from `sender`, with `sender` as fee recipient.
pub fn create_spot_limit_order(sender: &Address, order: &SpotLimitOrder) -> Result<Any, Error> {
    let id = order.market_id;
    let valid_id =
        id.len() == 66 && id.starts_with("0x") && id[2..].bytes().all(|b| b.is_ascii_hexdigit());
    if !valid_id {
        return Err(Error::InvalidMarketId(id.to_string()));
    }
    let msg = proto::MsgCreateSpotLimitOrder {
        sender: sender.to_string(),
        order: Some(proto::SpotOrder {
            market_id: id.to_ascii_lowercase(),
            order_info: Some(proto::OrderInfo {
                subaccount_id: sender.subaccount_id(order.subaccount_nonce),
                fee_recipient: sender.to_string(),
                price: dec::to_wire(order.price)?,
                quantity: dec::to_wire(order.quantity)?,
                cid: order.cid.unwrap_or_default().to_string(),
            }),
            order_type: order.side.order_type(),
            trigger_price: String::new(),
            expiration_block: 0,
        }),
    };
    Ok(Any {
        type_url: proto::MSG_CREATE_SPOT_LIMIT_ORDER.to_string(),
        value: msg.encode_to_vec(),
    })
}
