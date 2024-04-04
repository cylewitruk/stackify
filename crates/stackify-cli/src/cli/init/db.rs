use cliclack::{multi_progress, spinner, MultiProgress};
use color_eyre::Result;
use console::style;
use stackify_common::{FileType, ServiceType};

use crate::{
    cli::{context::CliContext, theme::ThemedObject},
    db::InsertServiceFile,
    includes::{BITCOIN_CONF, STACKS_NODE_CONF, STACKS_SIGNER_CONF},
};

struct InstallFile<'a> {
    filename: &'a str,
    description: &'a str,
    service_type: ServiceType,
    destination_dir: &'a str,
    default_contents: &'a [u8],
    file_type: FileType,
}

fn install_file(ctx: &CliContext, multi: &MultiProgress, file: InstallFile) -> Result<()> {
    // Insert Bitcoin Core configuration file template (for a miner)
    let spinner = multi.add(spinner());
    spinner.start("bitcoin.conf");
    if !ctx
        .db
        .check_if_service_type_file_exists(ServiceType::BitcoinMiner.into(), "bitcoin.conf")?
    {
        ctx.db.insert_service_file(InsertServiceFile {
            filename: file.filename.into(),
            description: file.description.into(),
            service_type_id: file.service_type as i32,
            destination_dir: file.destination_dir.into(),
            default_contents: file.default_contents.to_vec(),
            file_type_id: file.file_type as i32,
        })?;
        spinner.stop(format!("{} {}", style("✔").green(), "bitcoin.conf"));
    } else {
        spinner.cancel(format!(
            "{} {} {}",
            style("⊖").dim(),
            file.filename,
            style("skipped (already exists)").dimmed()
        ));
    }

    Ok(())
}

/// TODO: Super ugly... just doing this to get it done.
pub fn load_default_configuration_files(ctx: &CliContext) -> Result<()> {
    let multi = multi_progress("Default configuration files");

    // Insert Bitcoin Core configuration file template (for a miner)
    install_file(
        ctx,
        &multi,
        InstallFile {
            filename: "bitcoin.conf",
            description: "Bitcoin Core configuration file template",
            service_type: ServiceType::BitcoinMiner,
            destination_dir: "/home/stacks/.bitcoin",
            default_contents: BITCOIN_CONF,
            file_type: FileType::HandlebarsTemplate,
        },
    )?;

    install_file(
        ctx,
        &multi,
        InstallFile {
            filename: "stacks-node.toml",
            description: "Stacks Node configuration file template",
            service_type: ServiceType::StacksMiner,
            destination_dir: "/stacks/config/",
            default_contents: STACKS_NODE_CONF,
            file_type: FileType::HandlebarsTemplate,
        },
    )?;

    install_file(
        ctx,
        &multi,
        InstallFile {
            filename: "stacks-node.toml",
            description: "Stacks Node configuration file template",
            service_type: ServiceType::StacksFollower,
            destination_dir: "/stacks/config/",
            default_contents: STACKS_NODE_CONF,
            file_type: FileType::HandlebarsTemplate,
        },
    )?;

    install_file(
        ctx,
        &multi,
        InstallFile {
            filename: "stacks-signer.toml",
            description: "Stacks Signer configuration file template",
            service_type: ServiceType::StacksSigner,
            destination_dir: "/stacks/config/",
            default_contents: STACKS_SIGNER_CONF,
            file_type: FileType::HandlebarsTemplate,
        },
    )?;

    multi.stop();

    Ok(())
}
