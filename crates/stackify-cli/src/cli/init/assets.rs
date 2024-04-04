use std::{
    fs::{File, Permissions},
    io::Write,
    os::unix::fs::PermissionsExt as _,
};

use cliclack::multi_progress;
use color_eyre::Result;
use console::style;

use crate::{
    cli::{context::CliContext, theme::ThemedObject},
    includes::{
        BITCOIN_ENTRYPOINT, STACKIFY_BUILD_DOCKERFILE, STACKIFY_BUILD_ENTRYPOINT,
        STACKIFY_CARGO_CONFIG, STACKIFY_RUN_DOCKERFILE, STACKS_NODE_CONF, STACKS_SIGNER_CONF,
    },
};

pub fn copy_assets(ctx: &CliContext) -> Result<()> {
    let multi = multi_progress("Default assets");

    install_asset_executable(
        ctx,
        &multi,
        "build-entrypoint.sh",
        false,
        STACKIFY_BUILD_ENTRYPOINT,
    )?;
    install_asset_executable(
        ctx,
        &multi,
        "bitcoin-miner-entrypoint.sh",
        false,
        BITCOIN_ENTRYPOINT,
    )?;
    install_asset(
        ctx,
        &multi,
        "Dockerfile.build",
        false,
        STACKIFY_BUILD_DOCKERFILE,
    )?;
    install_asset(
        ctx,
        &multi,
        "Dockerfile.runtime",
        false,
        STACKIFY_RUN_DOCKERFILE,
    )?;
    install_asset(
        ctx,
        &multi,
        "cargo-config.toml",
        false,
        STACKIFY_CARGO_CONFIG,
    )?;
    install_asset(ctx, &multi, "stacks-node.toml.hbs", false, STACKS_NODE_CONF)?;
    install_asset(
        ctx,
        &multi,
        "stacks-signer.toml.hbs",
        false,
        STACKS_SIGNER_CONF,
    )?;

    multi.stop();

    Ok(())
}

/// Copies a file into the local Stackify assets directory and sets its executable permissions.
fn install_asset_executable(
    ctx: &CliContext,
    multi: &cliclack::MultiProgress,
    filename: &str,
    replace: bool,
    data: &[u8],
) -> Result<()> {
    install_asset(ctx, multi, filename, replace, data)?;
    let file = File::options()
        .write(true)
        .open(ctx.assets_dir.join(filename))?;
    file.set_permissions(Permissions::from_mode(0o744))?;
    file.sync_all()?;

    Ok(())
}

/// Copies a file into the local Stackify assets directory and sets its permissions to 644.
fn install_asset(
    ctx: &CliContext,
    multi: &cliclack::MultiProgress,
    filename: &str,
    replace: bool,
    data: &[u8],
) -> Result<()> {
    let spinner = multi.add(cliclack::spinner());
    spinner.start(filename);

    let mut file = match File::options()
        .create(true)
        .create_new(!replace)
        .write(true)
        .open(ctx.assets_dir.join(filename))
    {
        Ok(file) => file,
        Err(err) => {
            if err.kind() == std::io::ErrorKind::AlreadyExists {
                spinner.cancel(format!(
                    "{} {} {}",
                    style("⊖").dim(),
                    filename,
                    style("skipped (already exists)").dimmed()
                ));
                return Ok(());
            } else {
                return Err(err.into());
            }
        }
    };
    file.write_all(data)?;
    file.set_permissions(Permissions::from_mode(0o644))?;
    file.sync_all()?;

    spinner.stop(format!("{} {}", style("✔").green(), filename));

    Ok(())
}
