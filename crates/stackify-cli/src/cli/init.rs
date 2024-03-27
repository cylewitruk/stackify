use std::{borrow::Cow, fs::{File, Permissions}, io::{BufReader, Write}, os::unix::fs::PermissionsExt};

use clap::Args;
use color_eyre::Result;
use console::style;
use flate2::bufread::GzDecoder;
use futures_util::StreamExt;
use regex::Regex;
use stackify_common::{
    docker::{BuildStackifyBuildImage, BuildStackifyRuntimeImage},
    download::download_file,
    util::truncate,
};
use tar::Archive;

use crate::{
    cli::context::CliContext, includes::{BITCOIN_CONF, BITCOIN_ENTRYPOINT, STACKIFY_BUILD_DOCKERFILE, STACKIFY_BUILD_ENTRYPOINT, STACKIFY_CARGO_CONFIG, STACKIFY_RUN_DOCKERFILE}, util::{new_progressbar, print::print_bytes, progressbar::PbWrapper}
};

use super::theme::Inquire;

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
    #[arg(long, default_value = "false", required = false)]
    pub no_assets: bool,
}

pub fn exec(ctx: &CliContext, args: InitArgs) -> Result<()> {
    let disk_space_usage = match args.pre_compile {
        true => "~9GB",
        false => "~2.3GB",
    };

    if !args.no_build {
        let confirm = Inquire::new_confirm("This operation can take a while and consume a lot of disk space. Are you sure you want to continue?")
            .with_default(false)
            .with_help_message(&format!("Estimated disk space usage: {}", disk_space_usage))
            .prompt()?;

        if !confirm {
            println!("Aborted.");
            return Ok(());
        }
    }

    if !args.no_assets {
        copy_assets(ctx)?;
    }

    //ctx.docker.create_stackify_build_container(&ctx.bin_dir, STACKIFY_BUILD_ENTRYPOINT)?;

    if !args.no_download {
        // Download and extract Bitcoin Core and copy 'bitcoin-cli' and 'bitcoind'
        // to the bin directory.
        download_and_extract_bitcoin_core(ctx, &args.bitcoin_version)?;

        // Download Dasel (a jq-like tool for working with json,yaml,toml,xml,etc.).
        download_dasel(ctx, &args.dasel_version)?;
    }

    if !args.no_build {
        // Build the build image.
        build_build_image(ctx, &args.bitcoin_version, args.pre_compile)?;

        build_runtime_image(ctx)?;
    }

    load_default_configuration_files(ctx)?;

    Ok(())
}

fn download_dasel(ctx: &CliContext, version: &str) -> Result<()> {
    PbWrapper::new_spinner("Dasel").exec(|pb| {
        let mut download_size = 0;
        let mut progress = 0;
        pb.set_message("downloading...");
        let dasel_filename = "dasel_linux_amd64";
        let url = format!(
            "https://github.com/TomWright/dasel/releases/download/v{}/{}",
            version, dasel_filename
        );

        let dasel_bin = download_file(
            &url,
            &ctx.tmp_dir,
            |size| {
                download_size = size;
                pb.set_length(size);
                pb.set_message(format!("downloading... {}b", print_bytes(size)))
            },
            |chunk, total| {
                pb.inc(chunk);
                progress += chunk;
                pb.set_message(format!(
                    "downloading... {:.1}% ({}/{})",
                    progress as f32 / total as f32 * 100.0,
                    print_bytes(progress),
                    print_bytes(total)
                ))
            },
        )?;

        std::fs::copy(ctx.tmp_dir.join(&dasel_bin), ctx.bin_dir.join("dasel"))?;

        std::fs::remove_file(&dasel_bin)?;

        Ok(())
    })
}

fn download_and_extract_bitcoin_core(ctx: &CliContext, version: &str) -> Result<()> {
    let bitcoin_archive_filename = format!("bitcoin-{}-x86_64-linux-gnu.tar.gz", version);

    let bitcoin_url = format!(
        "https://bitcoincore.org/bin/bitcoin-core-{}/{}",
        version, bitcoin_archive_filename
    );

    // Download the file.
    PbWrapper::new_download_bar(u64::MAX, "Bitcoin Core").exec(|pb| {
        let mut download_size = 0;
        let mut progress = 0;
        pb.set_message("downloading...");
        let bitcoin_core_archive = download_file(
            &bitcoin_url,
            &ctx.tmp_dir,
            |size| {
                download_size = size;
                pb.set_length(size);
                pb.set_message(format!("downloading... {}b", print_bytes(size)))
            },
            |chunk, total| {
                pb.inc(chunk);
                progress += chunk;
                pb.set_message(format!(
                    "downloading... {:.1}% ({}/{})",
                    progress as f32 / total as f32 * 100.0,
                    print_bytes(progress),
                    print_bytes(total)
                ))
            },
        )?;

        pb.replace_with_spinner();

        pb.set_message("extracting archive...");
        let tmp_file = File::open(&bitcoin_core_archive)?;
        let gz = GzDecoder::new(BufReader::new(tmp_file));

        Archive::new(gz).unpack(&ctx.tmp_dir)?;

        pb.set_message("copying files...");
        let extracted_bin_dir = ctx.tmp_dir.join(format!("bitcoin-{}", version)).join("bin");

        std::fs::copy(
            extracted_bin_dir.join("bitcoin-cli"),
            ctx.bin_dir.join("bitcoin-cli"),
        )?;

        std::fs::copy(
            extracted_bin_dir.join("bitcoind"),
            ctx.bin_dir.join("bitcoind"),
        )?;

        std::fs::remove_dir_all(&ctx.tmp_dir)?;
        std::fs::create_dir(&ctx.tmp_dir)?;

        Ok(())
    })
}

fn build_build_image(ctx: &CliContext, bitcoin_version: &str, pre_compile: bool) -> Result<()> {
    let build = BuildStackifyBuildImage {
        user_id: ctx.user_id,
        group_id: ctx.group_id,
        bitcoin_version: bitcoin_version.into(),
        pre_compile,
        stackify_build_dockerfile: STACKIFY_BUILD_DOCKERFILE,
        stackify_cargo_config: STACKIFY_CARGO_CONFIG,
    };

    let regex = Regex::new(r#"^Step (\d+)\/(\d+) :(.*)$"#)?;
    let stream = ctx.docker.build_stackify_build_image(build)?;

    let pb = PbWrapper::new_progress_bar(0, "Stackify build image");

    tokio::runtime::Runtime::new()?.block_on(async {
        stream
            .for_each(|result| async {
                match result {
                    Ok(info) => {
                        if let Some(captures) = regex.captures(&info.message) {
                            let step: u64 = captures.get(1).unwrap().as_str().parse().unwrap();
                            let total: u64 = captures.get(2).unwrap().as_str().parse().unwrap();
                            let msg = captures.get(3).unwrap().as_str();
                            if pb.get_length() == Some(0) {
                                pb.set_length(total);
                            }
                            pb.inc(step - pb.get_position());
                            pb.set_message(Cow::Owned(truncate(msg, 70).to_owned()));
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            })
            .await
    });

    pb.finish_success();

    Ok(())
}

fn build_runtime_image(ctx: &CliContext) -> Result<()> {
    let build = BuildStackifyRuntimeImage {
        user_id: ctx.user_id,
        group_id: ctx.group_id,
        stackify_runtime_dockerfile: STACKIFY_RUN_DOCKERFILE
    };

    let regex = Regex::new(r#"^Step (\d+)\/(\d+) :(.*)$"#)?;
    let pb = new_progressbar(
        "{spinner:.dim.bold} runtime image: {wide_msg}",
        "Starting...",
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
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            })
            .await
    });
    pb.finish_and_clear();
    println!("{} Docker runtime image", style("✔️").green());

    Ok(())
}

fn create_runtime_container(ctx: &CliContext) -> Result<()> {
    todo!()
}

fn load_default_configuration_files(ctx: &CliContext) -> Result<()> {
    let bitcoin_conf = BITCOIN_CONF;
    todo!();
}

fn copy_assets(ctx: &CliContext) -> Result<()> {
    copy_executable(ctx, "build-entrypoint.sh", false, STACKIFY_BUILD_ENTRYPOINT)?;
    copy_executable(ctx, "bitcoin-miner-entrypoint.sh", false, BITCOIN_ENTRYPOINT)?;

    Ok(())
}

fn copy_executable(ctx: &CliContext, filename: &str, replace: bool, data: &[u8]) -> Result<()> {
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

fn copy_readable(ctx: &CliContext, filename: &str, data: &[u8]) -> Result<()> {
    let mut file = File::create_new(ctx.assets_dir.join(filename))?;
    file.write_all(data)?;
    file.set_permissions(Permissions::from_mode(0o644))?;
    file.sync_all()?;

    Ok(())
}