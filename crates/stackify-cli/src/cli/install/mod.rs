use clap::Args;
use cliclack::{multi_progress, outro, spinner};
use color_eyre::{eyre::bail, Result};

use crate::{
    cli::{
        context::CliContext,
        install::{db::load_default_configuration_params, docker::clean_images},
        log::clilog,
        theme::THEME,
        ABOUT,
    },
    docker::{opts::BuildImage, BuildResult},
    docker_api::opts::ImageBuildOpts,
};

use self::{
    assets::copy_assets, db::load_default_configuration_files, docker::build_image,
    downloads::download_and_extract_bitcoin_core,
};

use super::theme::ThemedObject;

mod assets;
mod db;
mod docker;
mod downloads;

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Specify the Bitcoin Core version to download.
    #[arg(long, default_value = "26.0", required = false)]
    pub bitcoin_version: String,

    /// Specify the Dasel version to download.
    #[arg(long, default_value = "2.7.0", required = false)]
    pub dasel_version: String,

    /// Specifies whether or not Cargo projects should be initalized (pre-compiled)
    /// in the build image. This ensures that all dependencies are already compiled,
    /// but results in a much larger image (c.a. 9GB vs 2.5GB). The trade-off is between size
    /// vs. build speed. If you plan on building new runtime binaries often, this
    /// may be a good option.
    #[arg(long, default_value = "false", required = false)]
    pub pre_compile: bool,
    /// Only download runtime binaries, do not build the images.
    #[arg(long, default_value = "false", required = false)]
    pub no_download: bool,
    /// Only build the images, do not download runtime binaries.
    #[arg(long, default_value = "false", required = false)]
    pub no_build: bool,
    /// Do not copy local assets to the assets directory.
    #[arg(long, default_value = "false", required = false)]
    pub no_assets: bool,
    #[arg(short, long, default_value = "false", required = false)]
    pub force: bool,
}

pub async fn exec(ctx: &CliContext, args: InitArgs) -> Result<()> {
    let disk_space_usage = match args.pre_compile {
        true => "~9GB",
        false => "~2.5GB",
    };

    println!("{}\n", ABOUT);

    cliclack::intro("Initialize Stackify".bold())?;
    cliclack::log::remark(
        "This operation will prepare your system for running Stackify.
It will download and build the necessary Docker images, create Stackify containers, download 
runtime binaries, initialize the database and copy assets to the appropriate directories.",
    )?;

    if !args.no_build {
        cliclack::log::warning(format!(
            "{} {} {}",
            "This can take several minutes and will consume".yellow(),
            disk_space_usage.red().bold(),
            "of disk space.".yellow()
        ))?;

        let confirm = cliclack::confirm("Are you sure you want to continue?").interact()?;

        if !confirm {
            cliclack::outro_cancel("Aborted by user")?;
            return Ok(());
        }
    }

    if !args.no_assets {
        copy_assets(ctx, args.force)?;
    }

    if !args.no_download {
        // Download and extract Bitcoin Core and copy 'bitcoin-cli' and 'bitcoind'
        // to the bin directory.
        download_and_extract_bitcoin_core(ctx, &args.bitcoin_version).await?;

        // Download Dasel (a jq-like tool for working with json,yaml,toml,xml,etc.).
        //download_dasel(ctx, &args.dasel_version).await?;
    }

    if !args.no_build {
        let multi = multi_progress("Build Docker images");

        // Clean up
        let clean_spinner = multi.add(spinner());
        clean_spinner.start("Cleaning up existing images...");
        clean_images(ctx).await?;
        clean_spinner.stop(format!("{} {}", "✔".green(), "Clean up existing images"));

        // Build the build image.
        clilog!("Building stackify build image");
        let build_spinner = multi.add(spinner());

        build_spinner.start("Preparing Stackify build image...");
        match build_image(
            ctx,
            &ImageBuildOpts::for_build_image(&ctx.host_dirs, args.pre_compile, args.force),
        )
        .await?
        {
            BuildResult::Success(id) => {
                build_spinner.stop(format!(
                    "{} {} {}",
                    THEME.read().unwrap().success_symbol(),
                    "Stackify build image",
                    id.dimmed()
                ));
            }
            BuildResult::Failed(error, message) => {
                build_spinner.stop(format!(
                    "{} {}",
                    THEME.read().unwrap().error_symbol(),
                    "Stackify build image"
                ));
                bail!("Build image failed: {} - {}", error, message);
            }
            BuildResult::Cancelled => {
                build_spinner.stop(format!(
                    "{} {}",
                    THEME.read().unwrap().skipped_symbol(),
                    "Stackify build image"
                ));
                bail!("Build image cancelled");
            }
        }

        // Build the runtime image.
        clilog!("Building stackify runtime image");
        let runtime_spinner = multi.add(spinner());
        runtime_spinner.start("Preparing Stackify runtime image...");
        match build_image(
            ctx,
            &ImageBuildOpts::for_runtime_image(&ctx.host_dirs, args.force),
        )
        .await?
        {
            BuildResult::Success(id) => {
                runtime_spinner.stop(format!(
                    "{} {} {}",
                    "✔".green(),
                    "Stackify runtime image",
                    id.dimmed()
                ));
            }
            BuildResult::Failed(error, message) => {
                runtime_spinner.stop(format!("{} {}", "✖".red(), "Failed to build runtime image"));
                bail!("Build image failed: {} - {}", error, message);
            }
            BuildResult::Cancelled => {
                runtime_spinner.stop(format!(
                    "{} {}",
                    "✖".red(),
                    "Operation was cancelled by the user"
                ));
                bail!("Build image cancelled");
            }
        }

        clilog!("Building stacks cli image");
        let stacks_cli_spinner = multi.add(spinner());
        stacks_cli_spinner.start("Preparing Stacks CLI image...");
        match build_image(
            ctx,
            &ImageBuildOpts::for_stacks_cli_image(&ctx.host_dirs, args.force),
        )
        .await?
        {
            BuildResult::Success(id) => {
                stacks_cli_spinner.stop(format!(
                    "{} {} {}",
                    "✔".green(),
                    "Stacks CLI image",
                    id.dimmed()
                ));
            }
            BuildResult::Failed(error, message) => {
                stacks_cli_spinner.stop(format!(
                    "{} {}",
                    "✖".red(),
                    "Failed to build stacks-cli image"
                ));
                bail!("Build image failed: {} - {}", error, message);
            }
            BuildResult::Cancelled => {
                stacks_cli_spinner.stop(format!(
                    "{} {}",
                    "✖".red(),
                    "Operation was cancelled by the user"
                ));
                bail!("Build image cancelled");
            }
        }

        multi.stop();
    }

    load_default_configuration_files(ctx, args.force)?;

    load_default_configuration_params(ctx, args.force)?;

    outro("Finished!".bold().green())?;

    Ok(())
}
