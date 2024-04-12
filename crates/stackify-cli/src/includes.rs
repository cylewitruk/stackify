// These are provided directly to the docker daemon for image building
pub const STACKIFY_BUILD_DOCKERFILE: &[u8] = include_bytes!("../assets/Dockerfile.build");
pub const STACKIFY_RUN_DOCKERFILE: &[u8] = include_bytes!("../assets/Dockerfile.runtime");
pub const STACKS_CLI_DOCKERFILE: &[u8] = include_bytes!("../assets/Dockerfile.stacks-cli");
pub const STACKIFY_BUILD_SETUP: &[u8] = include_bytes!("../assets/build-setup.sh");

// These are mounted to the container at creation time.
pub const STACKIFY_CARGO_CONFIG: &[u8] = include_bytes!("../assets/cargo-config.toml");
pub const STACKIFY_BUILD_ENTRYPOINT: &[u8] = include_bytes!("../assets/build-entrypoint.sh");
pub const BITCOIN_ENTRYPOINT: &[u8] = include_bytes!("../assets/bitcoin-entrypoint.sh");
pub const STACKS_NODE_ENTRYPOINT: &[u8] = include_bytes!("../assets/stacks-node-entrypoint.sh");

// These are loaded into the database
pub const BITCOIN_CONF: &[u8] = include_bytes!("../assets/bitcoin.conf.hbs");
pub const STACKS_NODE_CONF: &[u8] = include_bytes!("../assets/stacks-node.toml.hbs");
pub const STACKS_SIGNER_CONF: &[u8] = include_bytes!("../assets/stacks-signer.toml.hbs");
