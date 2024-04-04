use std::{
    fs::{File, Permissions},
    io::Write,
    os::unix::fs::PermissionsExt as _,
};

use color_eyre::Result;
use console::style;

use crate::{
    cli::context::CliContext,
    includes::{
        BITCOIN_ENTRYPOINT, STACKIFY_BUILD_DOCKERFILE, STACKIFY_BUILD_ENTRYPOINT,
        STACKIFY_CARGO_CONFIG, STACKIFY_RUN_DOCKERFILE, STACKS_NODE_CONF, STACKS_SIGNER_CONF,
    },
};

pub fn copy_assets(ctx: &CliContext) -> Result<()> {
    install_asset_executable(ctx, "build-entrypoint.sh", false, STACKIFY_BUILD_ENTRYPOINT)?;
    install_asset_executable(
        ctx,
        "bitcoin-miner-entrypoint.sh",
        false,
        BITCOIN_ENTRYPOINT,
    )?;
    install_asset(ctx, "Dockerfile.build", false, STACKIFY_BUILD_DOCKERFILE)?;
    install_asset(ctx, "Dockerfile.runtime", false, STACKIFY_RUN_DOCKERFILE)?;
    install_asset(ctx, "cargo-config.toml", false, STACKIFY_CARGO_CONFIG)?;
    install_asset(ctx, "stacks-node.toml.hbs", false, STACKS_NODE_CONF)?;
    install_asset(ctx, "stacks-signer.toml.hbs", false, STACKS_SIGNER_CONF)?;

    Ok(())
}

/// Copies a file into the local Stackify assets directory and sets its executable permissions.
fn install_asset_executable(
    ctx: &CliContext,
    filename: &str,
    replace: bool,
    data: &[u8],
) -> Result<()> {
    let mut file = match File::options()
        .create(true)
        .create_new(!replace)
        .write(true)
        .open(ctx.assets_dir.join(filename))
    {
        Ok(file) => file,
        Err(err) => {
            if err.kind() == std::io::ErrorKind::AlreadyExists {
                println!("{} already exists, skipping.", style(filename).dim());
                return Ok(());
            } else {
                return Err(err.into());
            }
        }
    };

    file.write_all(data)?;
    file.set_permissions(Permissions::from_mode(0o755))?;
    file.sync_all()?;

    Ok(())
}

/// Copies a file into the local Stackify assets directory and sets its permissions to 644.
fn install_asset(ctx: &CliContext, filename: &str, replace: bool, data: &[u8]) -> Result<()> {
    let mut file = match File::options()
        .create(true)
        .create_new(!replace)
        .write(true)
        .open(ctx.assets_dir.join(filename))
    {
        Ok(file) => file,
        Err(err) => {
            if err.kind() == std::io::ErrorKind::AlreadyExists {
                println!("{} already exists, skipping.", style(filename).dim());
                return Ok(());
            } else {
                return Err(err.into());
            }
        }
    };
    file.write_all(data)?;
    file.set_permissions(Permissions::from_mode(0o644))?;
    file.sync_all()?;

    Ok(())
}
