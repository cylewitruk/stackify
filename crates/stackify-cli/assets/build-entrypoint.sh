#! /usr/bin/env bash
# shellcheck disable=SC2059

cd /src || exit 1

git config --global --add safe.directory '*'

if [ "${BUILD_SBTC}" = "true" ]; then
  echo "Building SBTC" \
    && find ./ ! -name '.' -delete \
    && cp -rT /repos/sbtc /src \
    && git pull \
    && cargo --config /cargo-config.toml install --path sbtc-cli --root ./ \
    && mv -f ./bin/sbtc /out/sbtc \
    && echo "COMMIT_HASH=$(git rev-parse --short HEAD)"
fi

if [ "${BUILD_CLARINET}" = "true" ]; then
  echo "Building Clarinet" \
    && find ./ ! -name '.' -delete \
    && cp -rT /repos/clarinet /src \
    && git checkout main \
    && git pull \
    && git submodule update --recursive \
    && cargo --config /cargo-config.toml build --profile docker --bin clarinet \
    && mv -f /target/x86_64-unknown-linux-gnu/docker/clarinet /out/clarinet \
    && echo "COMMIT_HASH=$(git rev-parse --short HEAD)"
fi

if [ -n "${BUILD_NODE}" ]; then
  if [ -e "/out/stacks-node-${BUILD_NODE}" ]; then
    echo "stacks-node-${BUILD_NODE} already exists in the output directory. Skipping the build."
    exit 0
  fi
  echo "Building stacks-node binary"
  echo "Removing existing source files (if any)"
  find ./ ! -name '.' -delete 
  echo "Copying stacks-core source files"
  cp -rT /repos/stacks-core /src
  echo "Fetching the latest changes"
  git fetch --all
  echo "Checking out the specified tag/branch/commit (${BUILD_NODE})"
  git checkout "${BUILD_NODE}"
  echo "COMMIT_HASH=$(git log -n 1 --pretty=format:"%H" "${BUILD_NODE}")"
  echo "Pulling the latest changes"
  git pull
  echo "Building stacks-node"
  cargo --config /cargo-config.toml build --profile docker --package stacks-node --bin stacks-node
  echo "Moving the built binary to the output directory"
  mv -f /target/x86_64-unknown-linux-gnu/docker/stacks-node /out/stacks-node-"${BUILD_NODE}"
fi

if [ -n "${BUILD_CLI}" ]; then
  if [ -e "/out/blockstack-cli-${BUILD_CLI}" ]; then
    echo "blockstack-cli-${BUILD_CLI} already exists in the output directory. Skipping the build."
    exit 0
  fi
  echo "Building blockstack-cli binary"
  echo "Removing existing source files (if any)"
  find ./ ! -name '.' -delete 
  echo "Copying stacks-core source files"
  cp -rT /repos/stacks-core /src
  echo "Fetching the latest changes"
  git fetch --all
  echo "Checking out the specified tag/branch/commit (${BUILD_CLI})"
  git checkout "${BUILD_CLI}"
  echo "COMMIT_HASH=$(git log -n 1 --pretty=format:"%H" "${BUILD_CLI}")"
  echo "Pulling the latest changes"
  git pull
  echo "Building blockstack-cli"
  cargo --config /cargo-config.toml build --profile docker --package stackslib --bin blockstack-cli
  echo "Moving the built binary to the output directory"
  mv -f /target/x86_64-unknown-linux-gnu/docker/blockstack-cli /out/blockstack-cli-"${BUILD_CLI}"
fi

if [ -n "${BUILD_SIGNER}" ]; then
  if [ -e "/out/stacks-signer-${BUILD_SIGNER}" ]; then
    echo "stacks-signer-${BUILD_SIGNER} already exists in the output directory. Skipping the build."
    exit 0
  fi
  echo "Building Stacks signer"
  echo "Removing existing source files (if any)"
  find ./ ! -name '.' -delete 
  echo "Copying stacks-core source files"
  cp -rT /repos/stacks-core /src
  echo "Fetching the latest changes"
  git fetch --all
  echo "Checking out the specified tag/branch/commit (${BUILD_SIGNER})"
  git checkout "${BUILD_SIGNER}"
  echo "COMMIT_HASH=$(git log -n 1 --pretty=format:"%H" "${BUILD_SIGNER}")"
  echo "Pulling the latest changes"
  git pull
  echo "Building stacks-signer"
  cargo --config /cargo-config.toml build --profile docker --package stacks-signer --bin stacks-signer
  echo "Moving the built binary to the output directory"
  mv -f /target/x86_64-unknown-linux-gnu/docker/stacks-signer /out/stacks-signer-"${BUILD_SIGNER}"
fi