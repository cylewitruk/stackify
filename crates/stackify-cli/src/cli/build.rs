use std::{fs::File, io::BufReader};

use clap::Args;
use color_eyre::Result;
use console::style;
use flate2::bufread::GzDecoder;
use futures_util::StreamExt;
use regex::Regex;
use stackify_common::{
    docker::{BuildStackifyBuildImage, BuildStackifyRuntimeImage}, download::{download_dasel_binary, download_file}
};
use tar::Archive;

use crate::{context::CliContext, util::{new_progressbar, print::print_bytes, progressbar::PbWrapper}};

#[derive(Debug, Args)]
pub struct BuildArgs {
    #[arg(
        long,
        default_value = "26.0",
        required = false,
    )]
    pub bitcoin_version: String,
    #[arg(
        long,
        default_value = "2.7.0",
        required = false,
    )]
    pub dasel_version: String,
}

pub fn exec(ctx: &CliContext, args: BuildArgs) -> Result<()> {
    println!("Preparing Stackify artifacts...");

    // Download and extract Bitcoin Core and copy 'bitcoin-cli' and 'bitcoind' 
    // to the bin directory.
    download_and_extract_bitcoin_core(ctx, &args.bitcoin_version)?;

    // Download Dasel (a jq-like tool for working with json,yaml,toml,xml,etc.).
    download_dasel(ctx, &args.dasel_version)?;
    

    build_build_image(ctx, &args.bitcoin_version)?;

    build_runtime_image(ctx)?;
    

    Ok(())
}

fn download_dasel(ctx: &CliContext, version: &str) -> Result<()> {
    PbWrapper::new_spinner("Dasel")
        .exec(|pb| {
            let mut download_size = 0;
            let mut progress = 0;
            pb.set_message("downloading...");
            let dasel_filename = "dasel_linux_amd64";
            let url = format!("https://github.com/TomWright/dasel/releases/download/v{}/{}",
                version,
                dasel_filename);

            let dasel_bin = download_file(
                &url,
                &ctx.tmp_dir,
                |size| {
                    download_size = size;
                    pb.set_total_size(size);
                    pb.set_message(format!("downloading... {}b", print_bytes(size))
                    )
                },
                |chunk, total| {
                    pb.inc(chunk);
                    progress += chunk;
                    pb.set_message(format!("downloading... {:.1}% ({}/{})", 
                        progress as f32 / total as f32 * 100.0, 
                        print_bytes(progress), 
                        print_bytes(total))
                    )
                }
            )?;

            std::fs::copy(
                ctx.tmp_dir.join(&dasel_bin),
                ctx.bin_dir.join("dasel")
            )?;

            std::fs::remove_file(&dasel_bin)?;

            Ok(())
        })
}

fn download_and_extract_bitcoin_core(ctx: &CliContext, version: &str) -> Result<()> {
    let bitcoin_archive_filename = format!(
        "bitcoin-{}-x86_64-linux-gnu.tar.gz",
        version);

    let bitcoin_url = format!(
        "https://bitcoincore.org/bin/bitcoin-core-{}/{}",
        version,
        bitcoin_archive_filename);

    // Download the file.
    PbWrapper::new_progressbar(u64::MAX, "Bitcoin Core")
        .exec(|pb| {
            let mut download_size = 0;
            let mut progress = 0;
            pb.set_message("downloading...");
            let bitcoin_core_archive = download_file(
                &bitcoin_url,
                &ctx.tmp_dir, 
                |size| {
                    download_size = size;
                    pb.set_total_size(size);
                    pb.set_message(format!("downloading... {}b", print_bytes(size)))
                },
                |chunk, total| {
                    pb.inc(chunk);
                    progress += chunk;
                    pb.set_message(format!("downloading... {:.1}% ({}/{})", 
                        progress as f32 / total as f32 * 100.0, 
                        print_bytes(progress), 
                        print_bytes(total))
                    )
                }
            )?;

            pb.replace_with_spinner();

            pb.set_message("extracting archive...");
            let tmp_file = File::open(&bitcoin_core_archive)?;
            let gz = GzDecoder::new(BufReader::new(tmp_file));

            Archive::new(gz).unpack(&ctx.tmp_dir)?;

            pb.set_message("copying files...");
            let extracted_bin_dir = ctx.tmp_dir
                .join(format!("bitcoin-{}", version))
                .join("bin");

            std::fs::copy(
                extracted_bin_dir.join("bitcoin-cli"),
                ctx.bin_dir.join("bitcoin-cli")
            )?;

            std::fs::copy(
                extracted_bin_dir.join("bitcoind"),
                ctx.bin_dir.join("bitcoind")
            )?;

            std::fs::remove_dir_all(&ctx.tmp_dir)?;
            std::fs::create_dir(&ctx.tmp_dir)?;

            Ok(())
        })
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