//! Checks against outside references.

use base64::Engine as _;
use injective_client::msg::{self, coin, Side, SpotLimitOrder};
use injective_client::proto::{self, Any};
use injective_client::{
    dec, Address, Error, Fee, PrivateKey, PubKeyEncoding, SignerData, TxBuilder,
};
use prost::Message;

fn key(hex: &str) -> PrivateKey {
    let mut bytes = [0u8; 32];
    for (i, b) in bytes.iter_mut().enumerate() {
        *b = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).unwrap();
    }
    PrivateKey::from_bytes(&bytes).unwrap()
}

/// Trust Wallet Core's Injective signer test (tests/chains/Cosmos/NativeInjective/SignerTests.cpp,
/// Apache-2.0). Its comment links a Mainnet transaction whose hash is not the hash of these bytes,
/// and public nodes no longer keep that height, so this is checked as an independent
/// implementation's output only.
const TWC_KEY: &str = "9ee18daf8e463877aaf497282abc216852420101430482a28e246c179e2c5ef1";
const TWC_TX_BYTES: &str = "Co8BCowBChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5kEmwKKmluajEzdTZnN3ZxZ3cwNzRtZ21mMnplMmNhZHp2a3o5c25sd2NydHE4YRIqaW5qMXhtcGtteHI0YXMwMGVtMjN0YzJ6Z211eXkyZ3I0aDN3Z2NsNnZkGhIKA2luahILMTAwMDAwMDAwMDASngEKfgp0Ci0vaW5qZWN0aXZlLmNyeXB0by52MWJldGExLmV0aHNlY3AyNTZrMS5QdWJLZXkSQwpBBFoMa4O4vZgn5QcnDK20mbfjqQlSRvaiITKB94PYd8mLJWdCdBsGOfMXdo/k9MJ2JmDCESKDp2hdgVUH3uMikXMSBAoCCAEYARIcChYKA2luahIPMTAwMDAwMDAwMDAwMDAwELDbBhpAx2vkplmzeK7n3puCFGPWhLd0l/ZC/CYkGl+stH+3S3hiCvIe7uwwMpUlNaSwvT8HwF1kNUp+Sx2m0Uo1x5xcFw==";

#[test]
fn trust_wallet_core_transfer_byte_for_byte() {
    let key = key(TWC_KEY);
    let from = key.public_key().address();
    assert_eq!(
        from.to_string(),
        "inj13u6g7vqgw074mgmf2ze2cadzvkz9snlwcrtq8a"
    );
    let to = Address::from_bech32("inj1xmpkmxr4as00em23tc2zgmuyy2gr4h3wgcl6vd").unwrap();

    let signed = TxBuilder::new()
        .message(msg::bank_send(&from, &to, vec![coin("inj", "10000000000")]))
        .sign_direct(
            &key,
            PubKeyEncoding::Uncompressed,
            &SignerData {
                chain_id: "injective-1",
                account_number: 17396,
                sequence: 1,
            },
            &Fee {
                amount: vec![coin("inj", "100000000000000")],
                gas_limit: 110000,
            },
        );

    let expected = base64::engine::general_purpose::STANDARD
        .decode(TWC_TX_BYTES)
        .unwrap();
    assert_eq!(signed.tx_bytes, expected);
    assert_eq!(
        signed.hash(),
        "5CD8C1C9FF03F64946DF38F8F76D7CBDB1C10909D64D802BF6D38522EBED4CB6"
    );
}

#[test]
fn both_key_encodings_give_the_same_address() {
    let key = key(TWC_KEY);
    let public = key.public_key();
    assert_eq!(public.to_bytes(PubKeyEncoding::Compressed).len(), 33);
    assert_eq!(public.to_bytes(PubKeyEncoding::Uncompressed).len(), 65);
    let address = public.address();
    assert_eq!(Address::from_bech32(&address.to_string()).unwrap(), address);
}

#[test]
fn addresses_with_another_prefix_or_checksum_are_refused() {
    let good = "inj1xmpkmxr4as00em23tc2zgmuyy2gr4h3wgcl6vd";
    assert!(Address::from_bech32(good).is_ok());
    let mut bad_checksum = good.to_string();
    bad_checksum.pop();
    bad_checksum.push('q');
    assert!(Address::from_bech32(&bad_checksum).is_err());
    // A valid Bech32 string with the cosmos prefix.
    assert!(Address::from_bech32("cosmos1xmpkmxr4as00em23tc2zgmuyy2gr4h3wgcl6vd").is_err());
}

#[test]
fn legacy_dec_wire_form() {
    assert_eq!(dec::to_wire("1").unwrap(), "1000000000000000000");
    assert_eq!(dec::to_wire("1.5").unwrap(), "1500000000000000000");
    assert_eq!(dec::to_wire("0.001").unwrap(), "1000000000000000");
    assert_eq!(dec::to_wire("0.000000000000000001").unwrap(), "1");
    assert_eq!(dec::to_wire("0").unwrap(), "0");
    assert_eq!(dec::to_wire("000.0").unwrap(), "0");
    assert_eq!(dec::to_wire("1000").unwrap(), "1000000000000000000000");
    for bad in [
        "",
        ".5",
        "1.",
        "-1",
        "+1",
        "1e3",
        "1,5",
        " 1",
        "1.0000000000000000001",
        "1.2.3",
    ] {
        assert_eq!(
            dec::to_wire(bad),
            Err(Error::InvalidDecimal(bad.to_string())),
            "{bad:?}"
        );
    }
}

#[test]
fn spot_limit_order_fields() {
    let key = key(TWC_KEY);
    let sender = key.public_key().address();
    let market = "0x0611780BA69656949525013D947713300F56C37B6175E02F26BFFA495C3208FE";
    let any: Any = msg::create_spot_limit_order(
        &sender,
        &SpotLimitOrder {
            market_id: market,
            side: Side::SellPostOnly,
            price: "1000",
            quantity: "0.1",
            subaccount_nonce: 0,
            cid: Some("proto-1"),
        },
    )
    .unwrap();
    assert_eq!(
        any.type_url,
        "/injective.exchange.v2.MsgCreateSpotLimitOrder"
    );

    let msg = proto::MsgCreateSpotLimitOrder::decode(any.value.as_slice()).unwrap();
    assert_eq!(msg.sender, sender.to_string());
    let order = msg.order.unwrap();
    assert_eq!(order.market_id, market.to_ascii_lowercase());
    assert_eq!(order.order_type, 8);
    assert!(order.trigger_price.is_empty());
    let info = order.order_info.unwrap();
    // The address bytes, as an independent Bech32 decoder gives them, then a zero nonce.
    assert_eq!(
        info.subaccount_id,
        "0x8f348f300873fd5da36950b2ac75a26584584fee000000000000000000000000"
    );
    assert_eq!(
        sender.subaccount_id(1),
        "0x8f348f300873fd5da36950b2ac75a26584584fee000000000000000000000001"
    );
    assert_eq!(info.fee_recipient, sender.to_string());
    assert_eq!(info.price, "1000000000000000000000");
    assert_eq!(info.quantity, "100000000000000000");
    assert_eq!(info.cid, "proto-1");

    let bad = SpotLimitOrder {
        market_id: "0x1234",
        side: Side::Buy,
        price: "1",
        quantity: "1",
        subaccount_nonce: 0,
        cid: None,
    };
    assert!(matches!(
        msg::create_spot_limit_order(&sender, &bad),
        Err(Error::InvalidMarketId(_))
    ));
}
