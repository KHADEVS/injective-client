# injective-client

A Rust library to build and sign Injective transactions, including exchange
orders.

**Status: prototype.** Not audited, not on crates.io, and the API will change.
It has only been used on the testnet: don't use it with real funds yet.

## Why

Injective's official SDKs are in TypeScript, Python and Go. For Rust, its
documentation lists only low-level bindings and the CosmWasm libraries for
smart contracts. The Rust code that signs Injective transactions today (Gem
Wallet's core, fpco/cosmos-rs) covers simple transfers, not the exchange
module.

## What works

- Keys and addresses: `ethsecp256k1` keys, `inj1...` addresses, subaccount IDs.
- Transactions in `SIGN_MODE_DIRECT`, with the public key written compressed
  (as the official SDKs do) or uncompressed (as Trust Wallet Core does).
- Messages: bank transfers (`MsgSend`) and spot limit orders
  (`injective.exchange.v2.MsgCreateSpotLimitOrder`).
- The wire form of `LegacyDec` prices and quantities.

## How it is checked

| Check | What it proves |
|-------|----------------|
| Trust Wallet Core's Injective signer vector ([`tests/vectors.rs`](tests/vectors.rs)) | The same key, messages, fee and account give the same transaction bytes, byte for byte. |
| Testnet transfer with a compressed key: [`7BF26860...178F`](https://testnet.sentry.lcd.injective.network/cosmos/tx/v1beta1/txs/7BF26860B114A67E504C68BA1405E14DD85AB113CB0512D8EA73DD164D66178F), block 150128647 | The chain accepts the signature and the 33-byte key. |
| Testnet transfer with an uncompressed key: [`F7752CE0...23CB`](https://testnet.sentry.lcd.injective.network/cosmos/tx/v1beta1/txs/F7752CE0D8855E74AD0C72C0409AB0A33FB0539920F330AD7DAC23AA9B0723CB), block 150128661 | The chain accepts a 65-byte key in the signer info (on an account whose key it already knew). |
| Testnet spot order: [`533588B2...4C8C`](https://testnet.sentry.lcd.injective.network/cosmos/tx/v1beta1/txs/533588B225C1F542D33CDD3B9432BA059836C2B460F3C32EF1B95E0FB7FA4C8C), block 150128702 | The exchange module reads the order as sent: the INJ/USDT book shows a post-only sell of 0.01 at a price of 1000. |
| Unit tests | `LegacyDec` wire form, address checksum and prefix, order fields and subaccount IDs. |

The three testnet transactions were sent on 2026-11-24 to `injective-888`
(v1.21.0-beta.2) by [`examples/testnet.rs`](examples/testnet.rs), which checks
that the hash it computes is the one the node reports. They carry the memo
`injective-client prototype`.

## Two details that trip clients up

- `LegacyDec` values such as prices and quantities are decimals in JSON, but on
  the protobuf wire they are the value times 10^18, as an integer string:
  `1.5` is written `1500000000000000000`.
- An Injective key signs the Keccak-256 hash of the sign document, not its
  SHA-256 hash as on most Cosmos chains, and its address is the key's Ethereum
  address in Bech32.

## Use

```rust
use injective_client::msg::{self, coin, Side, SpotLimitOrder};
use injective_client::{Fee, PubKeyEncoding, SignerData, TxBuilder};

let sender = key.public_key().address();
let order = msg::create_spot_limit_order(
    &sender,
    &SpotLimitOrder {
        market_id: "0x0611780ba69656949525013d947713300f56c37b6175e02f26bffa495c3208fe",
        side: Side::SellPostOnly,
        price: "1000",
        quantity: "0.01",
        subaccount_nonce: 0,
        cid: None,
    },
)?;
let signed = TxBuilder::new().message(order).sign_direct(
    &key,
    PubKeyEncoding::Compressed,
    &SignerData { chain_id: "injective-888", account_number, sequence },
    &Fee { amount: vec![coin("inj", "125000000000000")], gas_limit: 780_000 },
);
// Broadcast signed.tx_bytes; signed.hash() is the transaction hash.
```

To send the same transactions to the testnet yourself:

```text
INJ_TESTNET_KEY_FILE=testnet.key cargo run --example testnet -- address
INJ_TESTNET_KEY_FILE=testnet.key cargo run --example testnet -- send <inj1 address> <amount> [compressed|uncompressed]
INJ_TESTNET_KEY_FILE=testnet.key cargo run --example testnet -- spot-order <market id> <buy|sell|buy-po|sell-po> <price> <quantity>
```

The key file is created if it does not exist. Fund its address from the
[testnet faucet](https://testnet.faucet.injective.network/).

## Next

- Every exchange message: derivative orders, batch updates, cancellations,
  subaccount transfers.
- Rounding prices and quantities to each market's tick sizes.
- Account, sequence and fee helpers.
- Indexer and chain streams.
- EIP-712 signing, checked against injective-ts.
- Vectors from sdk-go, a crates.io release, and automated updates when the
  chain's protobuf types change.

## License

MIT
