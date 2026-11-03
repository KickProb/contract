# KickProb — Soroban Contracts

Soroban (Stellar smart-contract) workspace that backs the two on-chain
pieces of [KickProb](https://kickprob.com):

1. **`subscription/`** — `KickProbSubscription`
   Accepts USDC (via the Stellar Asset Contract) and records paid plan
   entitlements (`BASIC`, `PREMIUM`, `VIP`). The backend (`KickProb/backend`)
   verifies the on-chain receipt before unlocking premium predictions.

2. **`receipt/`** — `KickProbReceipt`
   Lets the KickProb publisher commit a tamper-evident hash of each
   prediction batch (`publish_receipt`) at kickoff-freeze time, and later
   record the settled outcome rollup (`settle_receipt`). Both calls emit
   events that off-chain indexers and the public website can consume.

Companion repos:

- UI: https://github.com/KickProb/web
- API / worker / ML / research: https://github.com/KickProb/backend

## Why Stellar

- **USDC on Stellar** — low fees and ~5 second finality make small
  monthly subscription payments viable globally, with no NGN float.
- **Public receipts** — once a prediction batch hash is on chain, we can
  prove at any later time that a published pick was not edited after
  kickoff. Calibration claims become auditable.

## Build

Requires Rust stable + the `wasm32v1-none` target and the
[Stellar CLI](https://developers.stellar.org/docs/build/smart-contracts/getting-started/setup):

```bash
rustup target add wasm32v1-none
cargo install --locked stellar-cli
stellar contract build
```

## Test

```bash
cargo test
```

## Deploy (testnet)

See [`deploy-testnet.sh`](./deploy-testnet.sh). It expects
`STELLAR_NETWORK=testnet`, a funded identity, and the USDC Stellar Asset
Contract address on that network.

## Layout

```
Cargo.toml          # workspace
subscription/
  Cargo.toml
  src/lib.rs        # KickProbSubscription
receipt/
  Cargo.toml
  src/lib.rs        # KickProbReceipt
deploy-testnet.sh
```

## License

MIT
