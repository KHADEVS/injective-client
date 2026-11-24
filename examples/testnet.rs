//! Sends transactions to the Injective testnet (`injective-888`). Testnet only.
//!
//! The key is read from the file named by `INJ_TESTNET_KEY_FILE` (64 hex digits), and created
//! there if the file does not exist. Fund the printed address from the testnet faucet,
//! https://testnet.faucet.injective.network/, then:
//!
//! ```text
//! cargo run --example testnet -- address
//! cargo run --example testnet -- send <inj1 address> <amount in inj base units> [compressed|uncompressed]
//! cargo run --example testnet -- spot-order <market id> <buy|sell|buy-po|sell-po> <price> <quantity>
//! ```

use std::time::Duration;

use base64::Engine as _;
use injective_client::msg::{self, coin, Side, SpotLimitOrder};
use injective_client::proto::Any;
use injective_client::{Address, Fee, PrivateKey, PubKeyEncoding, SignerData, TxBuilder};
use serde_json::{json, Value};

const LCD: &str = "https://testnet.sentry.lcd.injective.network";
const CHAIN_ID: &str = "injective-888";
const EXPLORER: &str = "https://testnet.explorer.injective.network/transaction/";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let key = load_key()?;
    let me = key.public_key().address();

    match args.first().map(String::as_str) {
        Some("address") => {
            println!("{me}");
            println!(
                "balance: {}",
                get(&format!("/cosmos/bank/v1beta1/balances/{me}"))?["balances"]
            );
        }
        Some("send") if args.len() >= 3 => {
            let to = Address::from_bech32(&args[1])?;
            let encoding = match args.get(3).map(String::as_str) {
                None | Some("compressed") => PubKeyEncoding::Compressed,
                Some("uncompressed") => PubKeyEncoding::Uncompressed,
                Some(other) => return Err(format!("unknown key encoding {other}").into()),
            };
            let message = msg::bank_send(&me, &to, vec![coin("inj", &args[2])]);
            submit(&key, encoding, message)?;
        }
        Some("spot-order") if args.len() >= 5 => {
            let side = match args[2].as_str() {
                "buy" => Side::Buy,
                "sell" => Side::Sell,
                "buy-po" => Side::BuyPostOnly,
                "sell-po" => Side::SellPostOnly,
                other => return Err(format!("unknown side {other}").into()),
            };
            let message = msg::create_spot_limit_order(
                &me,
                &SpotLimitOrder {
                    market_id: &args[1],
                    side,
                    price: &args[3],
                    quantity: &args[4],
                    subaccount_nonce: 0,
                    cid: None,
                },
            )?;
            submit(&key, PubKeyEncoding::Compressed, message)?;
        }
        _ => {
            eprintln!("usage: address | send <to> <amount> [compressed|uncompressed] | spot-order <market id> <side> <price> <quantity>");
            std::process::exit(2);
        }
    }
    Ok(())
}

fn load_key() -> Result<PrivateKey> {
    let path = std::env::var("INJ_TESTNET_KEY_FILE")
        .map_err(|_| "set INJ_TESTNET_KEY_FILE to the path of the testnet key file")?;
    let hex = match std::fs::read_to_string(&path) {
        Ok(hex) => hex.trim().to_string(),
        Err(_) => {
            let key = PrivateKey::random(&mut rand_core::OsRng);
            let hex: String = key.to_bytes().iter().map(|b| format!("{b:02x}")).collect();
            std::fs::write(&path, &hex)?;
            eprintln!("created a new testnet key in {path}");
            hex
        }
    };
    if hex.len() != 64 {
        return Err("the key file must hold 64 hex digits".into());
    }
    let mut bytes = [0u8; 32];
    for (i, b) in bytes.iter_mut().enumerate() {
        *b = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16)?;
    }
    Ok(PrivateKey::from_bytes(&bytes)?)
}

fn get(path: &str) -> Result<Value> {
    Ok(ureq::get(&format!("{LCD}{path}"))
        .timeout(Duration::from_secs(30))
        .call()?
        .into_json()?)
}

fn post(path: &str, body: Value) -> Result<Value> {
    let response = ureq::post(&format!("{LCD}{path}"))
        .timeout(Duration::from_secs(30))
        .send_json(body);
    match response {
        Ok(r) => Ok(r.into_json()?),
        Err(ureq::Error::Status(code, r)) => {
            Err(format!("HTTP {code}: {}", r.into_string().unwrap_or_default()).into())
        }
        Err(e) => Err(e.into()),
    }
}

fn number(value: &Value) -> Result<u64> {
    Ok(value
        .as_str()
        .ok_or("expected a number in a string")?
        .parse()?)
}

/// Simulates the transaction for its gas, signs it with a fee at the current base fee,
/// broadcasts it and waits for it to be included.
fn submit(key: &PrivateKey, encoding: PubKeyEncoding, message: Any) -> Result<()> {
    let me = key.public_key().address();
    let account = get(&format!("/cosmos/auth/v1beta1/accounts/{me}"))?;
    let base = &account["account"]["base_account"];
    let signer = SignerData {
        chain_id: CHAIN_ID,
        account_number: number(&base["account_number"])?,
        sequence: number(&base["sequence"])?,
    };

    // The base fee is a decimal number of inj per gas unit; round it up.
    let base_fee = get("/injective/txfees/v1beta1/cur_eip_base_fee")?;
    let base_fee = base_fee["base_fee"]["base_fee"]
        .as_str()
        .ok_or("no base fee")?;
    let (integer, fraction) = base_fee.split_once('.').unwrap_or((base_fee, ""));
    let gas_price: u128 =
        integer.parse::<u128>()? + u128::from(fraction.bytes().any(|b| b != b'0'));

    let tx = TxBuilder::new()
        .message(message)
        .memo("injective-client prototype");
    let fee = |gas: u64| Fee {
        amount: vec![coin("inj", &(u128::from(gas) * gas_price).to_string())],
        gas_limit: gas,
    };
    let b64 = |bytes: &[u8]| base64::engine::general_purpose::STANDARD.encode(bytes);

    let draft = tx.sign_direct(key, encoding, &signer, &fee(400_000));
    let simulated = post(
        "/cosmos/tx/v1beta1/simulate",
        json!({ "tx_bytes": b64(&draft.tx_bytes) }),
    )?;
    let used = number(&simulated["gas_info"]["gas_used"])?;
    let gas = used + used / 3 + 20_000;
    println!("simulated: {used} gas, sending with a limit of {gas}");

    let signed = tx.sign_direct(key, encoding, &signer, &fee(gas));
    let hash = signed.hash();
    let response = post(
        "/cosmos/tx/v1beta1/txs",
        json!({ "tx_bytes": b64(&signed.tx_bytes), "mode": "BROADCAST_MODE_SYNC" }),
    )?;
    let code = response["tx_response"]["code"].as_u64().unwrap_or(u64::MAX);
    if code != 0 {
        return Err(format!("refused at broadcast: {}", response["tx_response"]).into());
    }
    if response["tx_response"]["txhash"].as_str() != Some(hash.as_str()) {
        return Err(format!(
            "the node reports another hash: {}",
            response["tx_response"]["txhash"]
        )
        .into());
    }
    println!("broadcast: {hash}");

    for _ in 0..30 {
        std::thread::sleep(Duration::from_secs(2));
        if let Ok(found) = get(&format!("/cosmos/tx/v1beta1/txs/{hash}")) {
            let r = &found["tx_response"];
            println!(
                "included at height {}, code {}, gas used {}",
                r["height"].as_str().unwrap_or("?"),
                r["code"],
                r["gas_used"].as_str().unwrap_or("?")
            );
            if r["code"].as_u64() != Some(0) {
                return Err(format!("failed in the block: {}", r["raw_log"]).into());
            }
            println!("{EXPLORER}{hash}");
            return Ok(());
        }
    }
    Err("not included after 60 seconds".into())
}
