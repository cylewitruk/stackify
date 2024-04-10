# Assets
This directory contains a number of assets which are compiled into the Stackify binary and then copied out to your Stackify home directory (typically `~/.stackify/assets`). These files are used in various ways when building and running containers.

### File Summary
- **Dockerfiles:**
  - `Dockerfile.build`: This Dockerfile is used to build the Stackify build image.
  - `Dockerfile.runtime`: This Dockerfile is used to build the Stackify runtime image.
- **Handlebars configuration file templates:**
  - `bitcoin.conf.hbs`: The Bitcoin node configuration template.
  - `stacks-node.toml.hbs`: Handlebars template for a Stacks node configuration file. This file is used both for miners and followers.
  - `stacks-signer.toml.hbs`: Handlebars template for a Sacks signer configuration file.
- **Scripts**
  - `build-entrypoint.sh`: Used by the Stackify build image for building Stacks binaries.
  - `bitcoin-entrypoint.sh`: Image entrypoint for a Bitcoin node. Whether or not the node acts as a miner is controlled with the `BITCOIN_MINER` environment variable, where a value of `true` will start a simulated miner.