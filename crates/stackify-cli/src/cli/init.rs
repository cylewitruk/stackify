use std::{
    fs::{File, Permissions},
    io::{BufReader, Write},
    os::unix::fs::PermissionsExt,
};

use clap::Args;
use color_eyre::Result;
use console::style;
use docker_api::{models::ImageBuildChunk, opts::ImageBuildOpts};
use flate2::bufread::GzDecoder;
use futures_util::StreamExt;
use regex::Regex;
use stackify_common::{
    docker::{BuildStackifyBuildImage, BuildStackifyRuntimeImage},
    download::download_file,
    FileType, ServiceType,
};
use tar::Archive;

use crate::{
    cli::context::CliContext,
    db::InsertServiceFile,
    includes::{
        BITCOIN_CONF, BITCOIN_ENTRYPOINT, STACKIFY_BUILD_DOCKERFILE, STACKIFY_BUILD_ENTRYPOINT,
        STACKIFY_CARGO_CONFIG, STACKIFY_RUN_DOCKERFILE, STACKS_NODE_CONF, STACKS_SIGNER_CONF,
    },
    util::new_progressbar,
};

use super::theme::ThemedObject;

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
        // Build the build image.
        cliclack::log::info("Building Docker images...")?;
        build_build_image(ctx, &args.bitcoin_version, args.pre_compile).await?;
        // Build the runtime image.
        build_runtime_image(ctx).await?;
    }

    load_default_configuration_files(ctx)?;

    Ok(())
}

/// Downloads the Dasel binary, which is a jq-like tool for working with json,
/// yaml, toml, xml, etc. and is useful to have in the runtime image for shell-
/// scripts to manipulate configuration files.
async fn download_dasel(ctx: &CliContext, version: &str) -> Result<()> {
    let multi = cliclack::progressbar_multi("Dasel");
    let dl = multi.add_downloadbar();
    let mut download_size = 0;
    let mut progress = 0;
    dl.start(100, "Downloading...");
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
            dl.set_length(size);
        },
        |chunk, _| {
            dl.increment(chunk);
            progress += chunk;
        },
    )
    .await?;

    std::fs::copy(ctx.tmp_dir.join(&dasel_bin), ctx.bin_dir.join("dasel"))?;

    std::fs::remove_file(&dasel_bin)?;

    dl.stop(format!("{} {}", style("✔").green(), "Download Dasel"))?;

    Ok(())
}

/// Downloads the specified version of Bitcore Core (full) and extracts the binaries
/// 'bitcoin-cli' and 'bitcoind' to the bin directory, which are needed for Bitcoin
/// miner+follower nodes.
async fn download_and_extract_bitcoin_core(ctx: &CliContext, version: &str) -> Result<()> {
    let bitcoin_archive_filename = format!("bitcoin-{}-x86_64-linux-gnu.tar.gz", version);

    let bitcoin_url = format!(
        "https://bitcoincore.org/bin/bitcoin-core-{}/{}",
        version, bitcoin_archive_filename
    );

    let multi = cliclack::progressbar_multi("Bitcoin Core");
    let dl = multi.add_downloadbar();

    let mut total_size = 0;
    let mut progress = 0;

    let bitcoin_core_archive = download_file(
        &bitcoin_url,
        &ctx.tmp_dir,
        |size| {
            total_size = size.clone();
            dl.start(size, "Downloading Bitoin Core...");
        },
        |chunk, _| {
            dl.increment(chunk);
            progress += chunk;
        },
    )
    .await?;
    dl.stop(format!(
        "{} {}",
        style("✔").green(),
        "Download Bitcoin Core"
    ))?;

    let mut unpack = cliclack::spinner();
    unpack.start("copying files...");

    let tmp_file = File::open(&bitcoin_core_archive)?;
    let gz = GzDecoder::new(BufReader::new(tmp_file));

    Archive::new(gz).unpack(&ctx.tmp_dir)?;
    unpack.stop(format!("{} {}", style("✔").green(), "Extract archive"));

    let mut cp = cliclack::spinner();
    cp.start("opying files...");
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
    cp.stop(format!("{} {}", style("✔").green(), "Copy binaries"));

    Ok(())
}

/// Builds the Stackfiy build image, which is used to compile the different versions
/// of runtime binaries.
async fn build_build_image(
    ctx: &CliContext,
    bitcoin_version: &str,
    pre_compile: bool,
) -> Result<()> {
    // let build = BuildStackifyBuildImage {
    //     user_id: ctx.user_id,
    //     group_id: ctx.group_id,
    //     bitcoin_version: bitcoin_version.into(),
    //     pre_compile,
    //     stackify_build_dockerfile: STACKIFY_BUILD_DOCKERFILE,
    //     stackify_cargo_config: STACKIFY_CARGO_CONFIG,
    // };

    let regex = Regex::new(r#"^Step (\d+)\/(\d+) :(.*)$"#)?;
    let mut spinner = cliclack::spinner();
    spinner.start("Building build image...");

    ctx.docker()
        .api()
        .images()
        .build(&ImageBuildOpts::default())
        .for_each(|result| async {
            match result {
                Ok(info) => {
                    match info {
                        ImageBuildChunk::Digest { aux, .. } => {
                            //spinner.start(format!("Digest: {}", aux.id));
                        }
                        ImageBuildChunk::Error {
                            error,
                            error_detail,
                        } => {
                            eprintln!("Error: {}", error);
                            eprintln!("Error Detail: {:?}", error_detail);
                        }
                        ImageBuildChunk::PullStatus {
                            status,
                            id,
                            progress,
                            progress_detail,
                        } => {
                            //spinner.start(format!("Pulling: {}", status));
                        }
                        ImageBuildChunk::Update { stream } => {
                            regex.captures(&stream).map(|captures| {
                                let step = captures.get(1).unwrap().as_str();
                                let total = captures.get(2).unwrap().as_str();
                                let msg = captures.get(3).unwrap().as_str();
                                //spinner.start(format!("[{}/{}]: {}", step, total, msg));
                            });
                        }
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        })
        .await;

    spinner.stop("Build image complete");

    Ok(())
}

/// Builds the Stackify runtime image, which is used to run the different
/// environment services.
async fn build_runtime_image(ctx: &CliContext) -> Result<()> {
    // let build = BuildStackifyRuntimeImage {
    //     user_id: ctx.user_id,
    //     group_id: ctx.group_id,
    //     stackify_runtime_dockerfile: STACKIFY_RUN_DOCKERFILE,
    // };

    let regex = Regex::new(r#"^Step (\d+)\/(\d+) :(.*)$"#)?;
    let mut spinner = cliclack::spinner();
    spinner.start("Building build image...");

    ctx.docker()
        .api()
        .images()
        .build(&ImageBuildOpts::default())
        .for_each(|result| async {
            match result {
                Ok(info) => {
                    match info {
                        ImageBuildChunk::Digest { aux, .. } => {
                            //spinner.start(format!("Digest: {}", aux.id));
                        }
                        ImageBuildChunk::Error {
                            error,
                            error_detail,
                        } => {
                            eprintln!("Error: {}", error);
                            eprintln!("Error Detail: {:?}", error_detail);
                        }
                        ImageBuildChunk::PullStatus {
                            status,
                            id,
                            progress,
                            progress_detail,
                        } => {
                            //spinner.start(format!("Pulling: {}", status));
                        }
                        ImageBuildChunk::Update { stream } => {
                            regex.captures(&stream).map(|captures| {
                                let step = captures.get(1).unwrap().as_str();
                                let total = captures.get(2).unwrap().as_str();
                                let msg = captures.get(3).unwrap().as_str();
                                //spinner.start(format!("[{}/{}]: {}", step, total, msg));
                            });
                        }
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        })
        .await;

    spinner.stop("Build image complete");

    Ok(())
}

/// TODO: Super ugly... just doing this to get it done.
fn load_default_configuration_files(ctx: &CliContext) -> Result<()> {
    // Insert Bitcoin Core configuration file template (for a miner)
    if !ctx
        .db
        .check_if_service_type_file_exists(ServiceType::BitcoinMiner.into(), "bitcoin.conf")?
    {
        ctx.db.insert_service_file(InsertServiceFile {
            filename: "bitcoin.conf".into(),
            description: "Bitcoin Core configuration file template".into(),
            service_type_id: ServiceType::BitcoinMiner as i32,
            destination_dir: "/home/stacks/.bitcoin".into(),
            default_contents: BITCOIN_CONF.as_bytes().to_vec(),
            file_type_id: FileType::HandlebarsTemplate as i32,
        })?;
    } else {
        println!("{} already exists, skipping.", style("bitcoin.conf").dim());
    }

    // Insert Bitcoin Core configuration file template (for a follower).
    if !ctx
        .db
        .check_if_service_type_file_exists(ServiceType::BitcoinFollower.into(), "bitcoin.conf")?
    {
        ctx.db.insert_service_file(InsertServiceFile {
            filename: "bitcoin.conf".into(),
            description: "Bitcoin Core configuration file template".into(),
            service_type_id: ServiceType::BitcoinFollower as i32,
            destination_dir: "/home/stacks/.bitcoin".into(),
            default_contents: BITCOIN_CONF.as_bytes().to_vec(),
            file_type_id: FileType::HandlebarsTemplate as i32,
        })?;
    } else {
        println!("{} already exists, skipping.", style("bitcoin.conf").dim());
    }

    // Insert Stacks Node configuration file template (for a miner).
    if !ctx
        .db
        .check_if_service_type_file_exists(ServiceType::StacksMiner.into(), "stacks-node.toml")?
    {
        ctx.db.insert_service_file(InsertServiceFile {
            filename: "stacks-node.toml".into(),
            description: "Stacks Node configuration file template".into(),
            service_type_id: ServiceType::StacksMiner as i32,
            destination_dir: "/stacks/config/".into(),
            default_contents: STACKS_NODE_CONF.as_bytes().to_vec(),
            file_type_id: FileType::HandlebarsTemplate as i32,
        })?;
    } else {
        println!(
            "{} already exists, skipping.",
            style("stacks-node.toml").dim()
        );
    }

    // Insert Stacks Node configuration file template (for a follower).
    if !ctx
        .db
        .check_if_service_type_file_exists(ServiceType::StacksFollower.into(), "stacks-node.toml")?
    {
        ctx.db.insert_service_file(InsertServiceFile {
            filename: "stacks-node.toml".into(),
            description: "Stacks Node configuration file template".into(),
            service_type_id: ServiceType::StacksFollower as i32,
            destination_dir: "/stacks/config/".into(),
            default_contents: STACKS_NODE_CONF.as_bytes().to_vec(),
            file_type_id: FileType::HandlebarsTemplate as i32,
        })?;
    } else {
        println!(
            "{} already exists, skipping.",
            style("stacks-node.toml").dim()
        );
    }

    // Insert Stacks Signer configuration file template.
    if !ctx
        .db
        .check_if_service_type_file_exists(ServiceType::StacksSigner.into(), "stacks-signer.toml")?
    {
        ctx.db.insert_service_file(InsertServiceFile {
            filename: "stacks-signer.toml".into(),
            description: "Stacks Signer configuration file template".into(),
            service_type_id: ServiceType::StacksSigner as i32,
            destination_dir: "/stacks/config/".into(),
            default_contents: STACKS_SIGNER_CONF.as_bytes().to_vec(),
            file_type_id: FileType::HandlebarsTemplate as i32,
        })?;
    } else {
        println!(
            "{} already exists, skipping.",
            style("stacks-signer.toml").dim()
        );
    }

    Ok(())
}

fn copy_assets(ctx: &CliContext) -> Result<()> {
    install_asset_executable(ctx, "build-entrypoint.sh", false, STACKIFY_BUILD_ENTRYPOINT)?;
    install_asset_executable(
        ctx,
        "bitcoin-miner-entrypoint.sh",
        false,
        BITCOIN_ENTRYPOINT,
    )?;

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
