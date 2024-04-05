#! /usr/bin/env bash

git clone https://github.com/stacks-network/stacks-core.git /repos/stacks-core
git clone https://github.com/stacks-network/sbtc.git /repos/sbtc
git clone https://github.com/hirosystems/clarinet.git --recursive /repos/clarinet

if [ "${PRE_COMPILE}" == "true" ]; then
  cd /repos/stacks-core \
    && cargo build --config /cargo-config.toml --profile docker --package stacks-node --bin stacks-node
fi

if [ "${PRE_COMPILE}" == "true" ]; then
  cd /repos/clarinet \
    && git submodule update --init --recursive \
    && cargo build --config /cargo-config.toml --profile docker
fi