use std::{fs::File, io::BufReader};

use cliclack::{outro, progress_bar, spinner};
use color_eyre::Result;
use console::style;
use flate2::bufread::GzDecoder;
use stackify_common::download::download_file;
use tar::Archive;

use crate::cli::{context::CliContext, theme::ThemedObject};

/// Downloads the Dasel binary, which is a jq-like tool for working with json,
/// yaml, toml, xml, etc. and is useful to have in the runtime image for shell-
/// scripts to manipulate configuration files.
pub async fn download_dasel(ctx: &CliContext, version: &str) -> Result<()> {
    let multi = cliclack::multi_progress("Dasel");
    let dl = multi.add(progress_bar(100).with_download_template());
    //let dl = multi.add_downloadbar();
    let mut download_size = 0;
    let mut progress = 0;

    dl.start("Downloading...");
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
            dl.inc(chunk);
            progress += chunk;
        },
    )
    .await?;
    dl.stop(format!("{} {}", style("✔").green(), "Download Dasel"));

    let cp = multi.add(spinner());
    cp.start("Installing...");
    std::fs::copy(ctx.tmp_dir.join(&dasel_bin), ctx.bin_dir.join("dasel"))?;
    std::fs::remove_file(&dasel_bin)?;
    cp.stop(format!(
        "{} {} {}",
        style("✔").green(),
        "Installed",
        format!("({}/)", ctx.bin_dir.display()).dimmed()
    ));

    multi.stop();

    Ok(())
}

/// Downloads the specified version of Bitcore Core (full) and extracts the binaries
/// 'bitcoin-cli' and 'bitcoind' to the bin directory, which are needed for Bitcoin
/// miner+follower nodes.
pub async fn download_and_extract_bitcoin_core(ctx: &CliContext, version: &str) -> Result<()> {
    let bitcoin_archive_filename = format!("bitcoin-{}-x86_64-linux-gnu.tar.gz", version);

    let bitcoin_url = format!(
        "https://bitcoincore.org/bin/bitcoin-core-{}/{}",
        version, bitcoin_archive_filename
    );

    let multi = cliclack::multi_progress("Bitcoin Core");
    let dl = multi.add(progress_bar(100).with_download_template());
    dl.start("Preparing to download...");

    let mut total_size = 0;
    let mut progress = 0;

    let bitcoin_core_archive = download_file(
        &bitcoin_url,
        &ctx.tmp_dir,
        |size| {
            total_size = size.clone();
            dl.set_message("Downloading...");
            dl.set_length(total_size);
        },
        |chunk, _| {
            dl.inc(chunk);
            progress += chunk;
        },
    )
    .await?;

    dl.stop(format!(
        "{} {}",
        style("✔").green(),
        "Download Bitcoin Core"
    ));

    let unpack = multi.add(spinner());
    unpack.start("Installing...");

    let tmp_file = File::open(&bitcoin_core_archive)?;
    let gz = GzDecoder::new(BufReader::new(tmp_file));

    Archive::new(gz).unpack(&ctx.tmp_dir)?;
    unpack.stop(format!("{} {}", style("✔").green(), "Extract archive"));

    let cp = multi.add(spinner());
    cp.start("Installed");
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
    cp.stop(format!(
        "{} {} {}",
        style("✔").green(),
        "Installed",
        format!("({}/)", ctx.bin_dir.display()).dimmed()
    ));

    multi.stop();

    Ok(())
}
