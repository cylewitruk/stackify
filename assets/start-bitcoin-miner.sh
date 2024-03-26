#!/bin/sh
mkdir -p /stacks/bitcoin

bitcoind >> /stacks/logs/bitcoind.log 2>&1 &

# Give bitcoind time to start before making RPC calls
sleep 1

# Import the 'stacks-regtest-miner' wallet
#bitcoin-cli restorewallet "stacks-regtest-miner" wallet.dat true
bitcoin-cli -rpcconnect=localhost -named createwallet wallet_name="default" descriptors=false load_on_startup=true

BTC_ADDRESS="bcrt1qng0yt0xgn40lykjj5q6xmgxfrqmdruy36kxgul"

# Generate 100 blocks to fund the wallet
bitcoin-cli -rpcconnect=localhost generatetoaddress 100 "$BTC_ADDRESS"

while :
do
        echo "Generate a new block $( date '+%d/%m/%Y %H:%M:%S' )"
        bitcoin-cli -rpcconnect=localhost generatetoaddress 1 $BTC_ADDRESS
        sleep 10
done