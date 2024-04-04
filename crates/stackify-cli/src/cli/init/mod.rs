use clap::Args;
use cliclack::{multi_progress, outro, spinner};
use color_eyre::Result;
use docker_api::opts::ImageBuildOpts;

use crate::{
    cli::{context::CliContext, ABOUT},
    docker::opts::BuildImage,
};

use self::{
    assets::copy_assets,
    db::load_default_configuration_files,
    docker::build_image,
    downloads::{download_and_extract_bitcoin_core, download_dasel},
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
}

pub async fn exec(ctx: &CliContext, args: InitArgs) -> Result<()> {
    let disk_space_usage = match args.pre_compile {
        true => "~9GB",
        false => "~2.3GB",
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
            "{}\n{} {}",
            "This operation can take a while and consume a lot of disk space.".yellow(),
            "Estimated disk space usage:",
            disk_space_usage.red().bold()
        ))?;

        let confirm = cliclack::confirm("Are you sure you want to continue?").interact()?;

        if !confirm {
            cliclack::outro_cancel("Aborted by user")?;
            return Ok(());
        }
    }

    if !args.no_assets {
        copy_assets(ctx)?;
    }

    if !args.no_download {
        // Download and extract Bitcoin Core and copy 'bitcoin-cli' and 'bitcoind'
        // to the bin directory.
        download_and_extract_bitcoin_core(ctx, &args.bitcoin_version).await?;

        // Download Dasel (a jq-like tool for working with json,yaml,toml,xml,etc.).
        download_dasel(ctx, &args.dasel_version).await?;
    }

    if !args.no_build {
        let multi = multi_progress("Build Docker images");

        // Build the build image.
        let build_spinner = multi.add(spinner());
        build_spinner.start("Building build image...");
        build_image(
            ctx,
            "stackify-build:latest",
            &ImageBuildOpts::for_build_image(&ctx.assets_dir),
        )
        .await?;
        build_spinner.stop(format!(
            "{} {} {}",
            "✔".green(),
            "Build image",
            "stackify-build:latest".dimmed()
        ));

        // Build the runtime image.
        let runtime_spinner = multi.add(spinner());
        runtime_spinner.start("Building runtime image...");
        build_image(
            ctx,
            "stackify-runtime:latest",
            &ImageBuildOpts::for_runtime_image(&ctx.assets_dir),
        )
        .await?;
        runtime_spinner.stop(format!(
            "{} {} {}",
            "✔".green(),
            "Runtime image",
            "stackify-runtime:latest".dimmed()
        ));

        multi.stop();
    }

    load_default_configuration_files(ctx)?;

    outro("Finished!".bold().green())?;

    Ok(())
}
