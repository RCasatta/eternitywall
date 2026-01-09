#!/usr/bin/env bash

# Dependencies check
for cmd in bitcoin-cli jq bc xxd; do
  if ! command -v $cmd &> /dev/null; then
    echo "Error: $cmd is not installed. Please add it to your environment."
    exit 1
  fi
done

# Configuration
FEE=0.00000250  # Manual fee in BTC

# 1. Get the message from the user
echo "Enter the message you want to publish on Eternity Wall:"
read -r USER_MESSAGE

# Prepend "EW " to the message
FULL_MESSAGE="EW $USER_MESSAGE"

# Convert text to Hex
HEX_MESSAGE=$(echo -n "$FULL_MESSAGE" | xxd -p | tr -d '\n')

# 2. Find a spendable UTXO
UTXO=$(bitcoin-cli listunspent | jq -r 'map(select(.spendable == true and .confirmations > 0))[0]')

if [ "$UTXO" == "null" ]; then
    echo "Error: No spendable, confirmed UTXOs found in wallet."
    exit 1
fi

TXID=$(echo "$UTXO" | jq -r '.txid')
VOUT=$(echo "$UTXO" | jq -r '.vout')
AMOUNT=$(echo "$UTXO" | jq -r '.amount')

# 3. Prepare Change Address and Calculation
CHANGE_ADDR=$(bitcoin-cli getrawchangeaddress)
CHANGE_AMOUNT=$(echo "$AMOUNT - $FEE" | bc -l | sed 's/^\./0./')

echo "------------------------------------------------"
echo "Message to Hex: $HEX_MESSAGE"
echo "Using UTXO: $TXID (vout $VOUT)"
echo "Input: $AMOUNT BTC | Fee: $FEE BTC | Change: $CHANGE_AMOUNT BTC"
echo "------------------------------------------------"

# 4. Create the raw transaction
# The 'data' key in the second argument creates the OP_RETURN output automatically
RAW_TX_HEX=$(bitcoin-cli createrawtransaction "[{\"txid\":\"$TXID\",\"vout\":$VOUT}]" "{\"data\":\"$HEX_MESSAGE\",\"$CHANGE_ADDR\":$CHANGE_AMOUNT}")

# 5. Sign the transaction
SIGNED_TX_JSON=$(bitcoin-cli signrawtransactionwithwallet "$RAW_TX_HEX")
SIGNED_HEX=$(echo "$SIGNED_TX_JSON" | jq -r '.hex')
COMPLETE=$(echo "$SIGNED_TX_JSON" | jq -r '.complete')

if [ "$COMPLETE" != "true" ]; then
    echo "Error: Transaction signing failed. Is your wallet locked?"
    exit 1
fi

# 6. Decode for review
echo "DECODING SIGNED TRANSACTION..."
bitcoin-cli decoderawtransaction "$SIGNED_HEX"
echo "------------------------------------------------"

# 7. Ask for broadcast
printf "Broadcast to network? (y/N): "
read -r CONFIRM

if [[ "$CONFIRM" =~ ^([yY][eE][sS]|[yY])$ ]]; then
    FINAL_TXID=$(bitcoin-cli sendrawtransaction "$SIGNED_HEX")
    echo "------------------------------------------------"
    echo "SUCCESS! Transaction broadcasted."
    echo "TXID: $FINAL_TXID"
    echo "View it: https://mempool.space/tx/$FINAL_TXID"
    echo "Check Eternity Wall: https://eternitywall.it/m/$FINAL_TXID"
else
    echo "Transaction discarded."
fi
