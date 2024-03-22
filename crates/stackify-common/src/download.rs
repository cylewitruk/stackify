use std::{fmt::Debug, fs::{self, File, Permissions}, io::{copy, BufReader, Cursor, Write}, os::unix::fs::PermissionsExt, path::Path};

use tar::Archive;
use color_eyre::eyre::Result;
use flate2::bufread::GzDecoder;
use flate2::bufread::MultiGzDecoder;
use flate2::bufread::ZlibDecoder;
use flate2::bufread::DeflateDecoder;

pub fn download_bitcoin_core_binaries<P: AsRef<Path> + Debug>(version: &str, tmp_dir: &P, dest_dir: &P) -> Result<()> {
    let filename = format!("bitcoin-{version}-x86_64-linux-gnu.tar.gz");
    let url = format!("https://bitcoincore.org/bin/bitcoin-core-{version}/{filename}");
    let tmp_file_path = tmp_dir.as_ref().join(filename);
    {
        let mut tmp_file = File::create(&tmp_file_path)?;
        let response = reqwest::blocking::get(&url)?;
        let mut content = Cursor::new(response.bytes()?);
        copy(&mut content, &mut tmp_file)?;
        tmp_file.flush()?;
    }
    
    let tmp_file = File::open(&tmp_file_path)?;
    let gz = GzDecoder::new(BufReader::new(tmp_file));

    //let tmp_dir = tempfile::tempdir()?;
    tar::Archive::new(gz).unpack(&tmp_dir)?;

    let bin_dir = tmp_dir.as_ref()
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
    let url = format!("https://github.com/TomWright/dasel/releases/download/v{version}/dasel_linux_amd64");
    let dest = dest_dir.as_ref().join("dasel");
    let mut dest_file = std::fs::File::create(&dest)?;
    let response = reqwest::blocking::get(&url)?;
    copy(&mut response.text()?.as_bytes(), &mut dest_file)?;
    
    set_executable(&dest)?;

    Ok(())
}

fn inner_copy<P: AsRef<Path>>(src: &P, dest: &P) -> Result<()> {
    let mut src_file = std::fs::File::open(src)?;
    let mut dest_file = std::fs::File::create(dest)?;

    copy(&mut src_file, &mut dest_file)?;

    Ok(())   
}

fn set_executable<P: AsRef<Path>>(path: &P) -> Result<()> {
    let perm = Permissions::from_mode(0o744);
    let file = File::open(path)?;
    file.set_permissions(perm)?;
    Ok(())
}