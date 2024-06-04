#! /usr/bin/env bash

echo "Starting stacks-signer"

start_stacks_signer() {
  stacks-signer-"${VERSION}" run --config /opt/stackify/config/stacks-signer.toml
}

start_stacks_signer