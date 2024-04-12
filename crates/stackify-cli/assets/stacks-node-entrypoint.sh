#! /usr/bin/env bash

start_stacks_node() {
  stacks-node-"${VERSION}" start --config /opt/stackify/config/stacks-node.toml
}

if [ "$MINER" == "false" ]; then
  while : ; do
    sleep 1
  done
elif [ "$MINER" == "true" ]; then
  start_stacks_node
fi
