#! /usr/bin/env bash
# shellcheck disable=SC2059

cd /src || exit 1

if [ "${BUILD_SBTC}" = "true" ]; then
  echo "Building SBTC" \
    && find ./ ! -name '.' -delete \
    && cp -rT ~/repos/sbtc /src \
    && git pull \
    && cargo --config ~/.cargo/config.toml install --path sbtc-cli --root ./ \
    && mv -f ./bin/sbtc /out/sbtc
fi

if [ "${BUILD_CLARINET}" = "true" ]; then
  echo "Building Clarinet" \
    && find ./ ! -name '.' -delete \
    && cp -rT ~/repos/clarinet /src \
    && git checkout main \
    && git pull \
    && git submodule update --recursive \
    && cargo --config ~/.cargo/config.toml build --profile docker --bin clarinet \
    && mv -f /target/x86_64-unknown-linux-gnu/docker/clarinet /out/clarinet
fi

if [ "${BUILD_STACKS}" = "true" ]; then
  echo "Building Stacks binaries" \
    && find ./ ! -name '.' -delete \
    && cp -rT ~/repos/stacks-core /src \
    && git checkout "${STACKS_BRANCH_TAG_REV}" \
    && cargo --config ~/.cargo/config.toml build --profile docker --package stacks-node --bin stacks-node \
    && mv -f /target/x86_64-unknown-linux-gnu/docker/stacks-node /out/stacks-node-"${STACKS_BRANCH_TAG_REV}"
fi