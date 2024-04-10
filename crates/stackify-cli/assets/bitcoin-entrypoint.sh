#!/bin/sh

bitcoin_cli() {
  bitcoin-cli-"${BITCOIN_VERSION}" -conf=/opt/bitcoin/bitcoin.conf -rpcconnect=127.0.0.1 "$@"
}

bitcoind() {
  bitcoind-"${BITCOIN_VERSION}" -conf=/opt/bitcoin/bitcoin.conf
}

bitcoind >> /var/log/stackify/bitcoind.log 2>&1 &

if [ "${BITCOIN_MINER}" = "true" ]; then
  # Give bitcoind time to start before making RPC calls
  sleep 1

  # Import the 'stacks-regtest-miner' wallet
  #bitcoin-cli restorewallet "stacks-regtest-miner" wallet.dat true
  bitcoin_cli -named createwallet wallet_name="default" descriptors=false load_on_startup=true

  BTC_ADDRESS="bcrt1qng0yt0xgn40lykjj5q6xmgxfrqmdruy36kxgul"

  # Generate 100 blocks to fund the wallet
  bitcoin_cli generatetoaddress 100 "$BTC_ADDRESS"

  while :
  do
    echo "Generate a new block $( date '+%d/%m/%Y %H:%M:%S' )"
    bitcoin_cli generatetoaddress 1 $BTC_ADDRESS
    sleep 10
  done
else
  while : ; do sleep 1; done
fi
