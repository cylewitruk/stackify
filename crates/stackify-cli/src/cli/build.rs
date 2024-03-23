use std::time::Duration;

use clap::Args;
use color_eyre::Result;
use console::style;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use stackify_common::{
    docker::BuildStackifyBuildImage, docker::BuildStackifyRuntimeImage, 
    download::{download_bitcoin_core_binaries, download_dasel_binary}
};

use crate::{context::CliContext, util::new_progressbar};

#[derive(Debug, Args)]
pub struct BuildArgs {
    #[arg(
        short = 'b',
        long,
        default_value = "26.0",
        required = false,
    )]
    pub bitcoin_version: String
}

pub fn exec(ctx: &CliContext, args: BuildArgs) -> Result<()> {
    println!("Preparing Stackify artifacts...");

    download_bitcoin(&ctx, &args.bitcoin_version)?;

    download_dasel(&ctx)?;

    build_build_image(ctx, &args.bitcoin_version)?;

    build_runtime_image(ctx)?;
    

    Ok(())
}

fn download_bitcoin(ctx: &CliContext, version: &str) -> Result<()> {
    let pb = new_progressbar("{spinner:.dim.bold} download: {wide_msg}", "Downloading bitcoin core binaries...");
    download_bitcoin_core_binaries(version, &ctx.tmp_dir, &ctx.bin_dir)?;
    pb.finish_and_clear();
    println!(
        "{} Bitcoin Core binaries",
        style("✔️").green());

    Ok(())
}

fn download_dasel(ctx: &CliContext) -> Result<()> {
    let pb = new_progressbar("{spinner:.dim.bold} download: {wide_msg}", "Downloading dasel...");
    download_dasel_binary("2.7.", &ctx.bin_dir)?;
    pb.finish_and_clear();
    println!(
        "{} Dasel",
        style("✔️").green()
    );
    Ok(())
}

fn build_build_image(ctx: &CliContext, bitcoin_version: &str) -> Result<()> {
    let build = BuildStackifyBuildImage {
        user_id: ctx.user_id,
        group_id: ctx.group_id,
        bitcoin_version: bitcoin_version.into(),
    };

    let regex = Regex::new(r#"^Step (\d+)\/(\d+) :(.*)$"#)?;
    let pb = new_progressbar(
        "{spinner:.dim.bold} build image: {wide_msg}", 
        "Starting..."
    );

    let stream = ctx.docker.build_stackify_build_image(build)?;

    tokio::runtime::Runtime::new()?.block_on(async {
        stream
            .for_each(|result| async {
                match result {
                    Ok(info) => {
                        regex.captures(&info.message).map(|captures| {
                            let step = captures.get(1).unwrap().as_str();
                            let total = captures.get(2).unwrap().as_str();
                            let msg = captures.get(3).unwrap().as_str();
                            pb.set_message(format!("[{}/{}]: {}", step, total, msg));
                        });
                    },
                    Err(e) => eprintln!("Error: {}", e),
                }
            })
            .await
    });
    pb.finish_and_clear();
    println!(
        "{} Docker build image",
        style("✔️").green()
    );

    Ok(())
}

fn build_runtime_image(ctx: &CliContext) -> Result<()> {
    let build = BuildStackifyRuntimeImage {
        user_id: ctx.user_id,
        group_id: ctx.group_id,
    };

    let regex = Regex::new(r#"^Step (\d+)\/(\d+) :(.*)$"#)?;
    let pb = new_progressbar(
        "{spinner:.dim.bold} runtime image: {wide_msg}", 
        "Starting..."
    );

    let stream = ctx.docker.build_stackify_runtime_image(build)?;

    tokio::runtime::Runtime::new()?.block_on(async {
        stream
            .for_each(|result| async {
                match result {
                    Ok(info) => {
                        regex.captures(&info.message).map(|captures| {
                            let step = captures.get(1).unwrap().as_str();
                            let total = captures.get(2).unwrap().as_str();
                            let msg = captures.get(3).unwrap().as_str();
                            pb.set_message(format!("[{}/{}]: {}", step, total, msg));
                        });
                    },
                    Err(e) => eprintln!("Error: {}", e),
                }
            })
            .await
    });
    pb.finish_and_clear();
    println!(
        "{} Docker runtime image",
        style("✔️").green()
    );

    Ok(())
}