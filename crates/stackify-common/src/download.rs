use std::{
    fmt::Debug,
    fs::{self, File, Permissions},
    io::{copy, BufReader, Write},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    time::Duration,
};

use color_eyre::eyre::{eyre, Result};
use flate2::bufread::GzDecoder;
use reqwest::{header, Client, Url};
use tar::Archive;

pub async fn download_file<P: AsRef<Path> + Debug>(
    url_str: &str,
    tmp_dir: &P,
    dl_start: impl FnOnce(u64),
    mut dl_chunk: impl FnMut(u64, u64),
) -> Result<PathBuf> {
    let url = Url::parse(&url_str)?;
    let filename = url
        .path_segments()
        .unwrap()
        .last()
        .expect("Could not determine filename.");

    let tmp_file_path = tmp_dir.as_ref().join(filename);

    let download_size = get_download_size(url).await?;
    dl_start(download_size);

    let mut tmp_file = File::create(&tmp_file_path)?;

    let client = Client::builder()
        .timeout(Duration::from_secs(500))
        .build()?;

    let request = client.get(url_str);
    let mut download = request.send().await?;
    while let Some(chunk) = download.chunk().await? {
        dl_chunk(chunk.len() as u64, download_size);
        tmp_file.write(&chunk)?;
    }

    tmp_file.flush()?;

    Ok(tmp_file_path)
}

pub async fn download_bitcoin_core_binaries<P: AsRef<Path> + Debug>(
    version: &str,
    tmp_dir: &P,
    dest_dir: &P,
    dl_start: impl FnOnce(u64),
    mut dl_chunk: impl FnMut(u64, u64),
    dl_finished: impl FnOnce(),
) -> Result<()> {
    let filename = format!("bitcoin-{version}-x86_64-linux-gnu.tar.gz");
    let url_str = format!("https://bitcoincore.org/bin/bitcoin-core-{version}/{filename}");

    let url = Url::parse(&url_str)?;

    let tmp_file_path = tmp_dir.as_ref().join(filename);

    let download_size = get_download_size(url).await?;
    dl_start(download_size);

    let mut tmp_file = File::create(&tmp_file_path)?;

    let client = Client::builder()
        .timeout(Duration::from_secs(500))
        .build()?;

    let request = client.get(&url_str);
    let mut download = request.send().await?;
    while let Some(chunk) = download.chunk().await? {
        dl_chunk(chunk.len() as u64, download_size);
        tmp_file.write(&chunk)?;
    }

    tmp_file.flush()?;
    dl_finished();

    let tmp_file = File::open(&tmp_file_path)?;
    let gz = GzDecoder::new(BufReader::new(tmp_file));

    //let tmp_dir = tempfile::tempdir()?;
    Archive::new(gz).unpack(&tmp_dir)?;

    let bin_dir = tmp_dir
        .as_ref()
        .join(format!("bitcoin-{version}"))
        .join("bin");

    let bitcoin_cli_src = bin_dir.join("bitcoin-cli");
    let bitcoin_cli_dest = dest_dir.as_ref().join("bitcoin-cli");
    inner_copy(&bitcoin_cli_src, &bitcoin_cli_dest)?;
    let bitcoind_src = bin_dir.join("bitcoind");
    let bitcoind_dest = dest_dir.as_ref().join("bitcoind");
    inner_copy(&bitcoind_src, &bitcoind_dest)?;

    fs::remove_dir_all(&tmp_dir)?;

    set_executable(&bitcoin_cli_dest)?;
    set_executable(&bitcoind_dest)?;

    Ok(())
}

pub fn download_dasel_binary<P: AsRef<Path>>(version: &str, dest_dir: P) -> Result<()> {
    let url = format!(
        "https://github.com/TomWright/dasel/releases/download/v{version}/dasel_linux_amd64"
    );
    let dest = dest_dir.as_ref().join("dasel");
    let mut dest_file = std::fs::File::create(&dest)?;
    let response = reqwest::blocking::get(&url)?;
    copy(&mut response.text()?.as_bytes(), &mut dest_file)?;

    set_executable(&dest)?;

    Ok(())
}

pub fn inner_copy<P: AsRef<Path>>(src: &P, dest: &P) -> Result<()> {
    let mut src_file = std::fs::File::open(src)?;
    let mut dest_file = std::fs::File::create(dest)?;

    copy(&mut src_file, &mut dest_file)?;

    Ok(())
}

pub fn set_executable<P: AsRef<Path>>(path: &P) -> Result<()> {
    let perm = Permissions::from_mode(0o744);
    let file = File::open(path)?;
    file.set_permissions(perm)?;
    Ok(())
}

// Help from https://github.com/benkay86/async-applied/blob/master/indicatif-reqwest-tokio/src/bin/indicatif-reqwest-tokio-single.rs
async fn get_download_size(url: Url) -> Result<u64> {
    let client = reqwest::Client::new();
    // We need to determine the file size before we download so we can create a ProgressBar
    // A Header request for the CONTENT_LENGTH header gets us the file size
    let download_size = {
        let resp = client.head(url.as_str()).send().await?;
        if resp.status().is_success() {
            resp.headers() // Gives is the HeaderMap
                .get(header::CONTENT_LENGTH) // Gives us an Option containing the HeaderValue
                .and_then(|ct_len| ct_len.to_str().ok()) // Unwraps the Option as &str
                .and_then(|ct_len| ct_len.parse().ok()) // Parses the Option as u64
                .unwrap_or(0) // Fallback to 0
        } else {
            // We return an Error if something goes wrong here
            return Err(
                eyre!("Couldn't download URL: {}. Error: {:?}", url, resp.status(),).into(),
            );
        }
    };

    Ok(download_size as u64)
}
