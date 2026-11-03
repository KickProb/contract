#!/usr/bin/env bash
# Deploy KickProb contracts to the Stellar testnet.
#
# Prereqs:
#   - stellar CLI installed (https://developers.stellar.org/docs/build/smart-contracts/getting-started/setup)
#   - A funded identity:   stellar keys generate deployer --network testnet
#   - USDC SAC address for the chosen network, exported as USDC_SAC
#   - Treasury account address exported as TREASURY
#   - Backend publisher key address exported as OPERATOR
#
# Usage:
#   STELLAR_IDENTITY=deployer USDC_SAC=... TREASURY=... OPERATOR=... ./deploy-testnet.sh
set -euo pipefail

: "${STELLAR_IDENTITY:?set to a stellar CLI identity}"
: "${USDC_SAC:?set to the USDC Stellar Asset Contract address on testnet}"
: "${TREASURY:?set to the treasury account address}"
: "${OPERATOR:?set to the KickProb backend publisher address}"
NETWORK="${STELLAR_NETWORK:-testnet}"

echo "[1/4] stellar contract build"
stellar contract build

WASM_SUB="target/wasm32v1-none/release/kickprob_subscription.wasm"
WASM_REC="target/wasm32v1-none/release/kickprob_receipt.wasm"

echo "[2/4] deploy KickProbSubscription"
SUB_ID=$(stellar contract deploy \
  --source "$STELLAR_IDENTITY" --network "$NETWORK" \
  --wasm "$WASM_SUB")
echo "  subscription=$SUB_ID"

ADMIN=$(stellar keys address "$STELLAR_IDENTITY")

stellar contract invoke --id "$SUB_ID" --source "$STELLAR_IDENTITY" --network "$NETWORK" -- \
  initialize --admin "$ADMIN" --usdc "$USDC_SAC" --treasury "$TREASURY"

echo "[3/4] deploy KickProbReceipt"
REC_ID=$(stellar contract deploy \
  --source "$STELLAR_IDENTITY" --network "$NETWORK" \
  --wasm "$WASM_REC")
echo "  receipt=$REC_ID"

stellar contract invoke --id "$REC_ID" --source "$STELLAR_IDENTITY" --network "$NETWORK" -- \
  initialize --admin "$ADMIN" --operator "$OPERATOR"

echo "[4/4] done"
printf 'KICKPROB_SUBSCRIPTION_ID=%s\nKICKPROB_RECEIPT_ID=%s\n' "$SUB_ID" "$REC_ID"
