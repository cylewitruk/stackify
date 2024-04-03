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

if [ -n "${BUILD_STACKS}" ]; then
  echo "Building Stacks binaries"
  while : ; do echo "sleep 5" && sleep 5; done
  echo "Removing existing source files (if any)"
  find ./ ! -name '.' -delete 
  echo "Copying stacks-core source files"
  cp -rT ~/repos/stacks-core /src 
  echo "Checking out the specified tag/branch/commit (${BUILD_STACKS})"
  git checkout "${BUILD_STACKS}" 
  echo "Pulling the latest changes"
  git pull
  echo "Building stacks-node"
  cargo --config ~/.cargo/config.toml build --profile docker --package stacks-node --bin stacks-node
  echo "Moving the built binary to the output directory"
  mv -f /target/x86_64-unknown-linux-gnu/docker/stacks-node /out/stacks-node-"${BUILD_STACKS}"
fi