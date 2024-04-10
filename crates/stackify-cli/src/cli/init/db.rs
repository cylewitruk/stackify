use cliclack::{multi_progress, spinner, MultiProgress};
use color_eyre::Result;
use console::style;
use stackify_common::{FileType, ServiceType};

use crate::{
    cli::{context::CliContext, log::clilog, theme::ThemedObject},
    db::InsertServiceFile,
    includes::{BITCOIN_CONF, STACKS_NODE_CONF, STACKS_SIGNER_CONF},
};

struct InstallFile<'a> {
    filename: &'a str,
    description: &'a str,
    service_type: ServiceType,
    service_type_name: String,
    destination_dir: &'a str,
    default_contents: &'a [u8],
    file_type: FileType,
}

/// Helper method to install an configuration file into the database
fn install_file(
    ctx: &CliContext,
    multi: &MultiProgress,
    file: InstallFile,
    force: bool,
) -> Result<()> {
    // Insert Bitcoin Core configuration file template (for a miner)
    let spinner = multi.add(spinner());
    spinner.start(file.filename);
    let service_type_id = file.service_type as i32;
    clilog!(
        "Inserting file: {} [{}], force={}",
        file.filename,
        file.service_type_name,
        force
    );
    if force
        || !ctx
            .db
            .check_if_service_type_file_exists(service_type_id, file.filename)?
    {
        ctx.db.insert_service_file(InsertServiceFile {
            filename: file.filename.into(),
            description: file.description.into(),
            service_type_id,
            destination_dir: file.destination_dir.into(),
            default_contents: file.default_contents.to_vec(),
            file_type_id: file.file_type as i32,
        })?;
        spinner.stop(format!(
            "{} {} {}",
            style("✔").green(),
            file.filename,
            format!("[{}]", file.service_type_name).dimmed()
        ));
    } else {
        spinner.cancel(format!(
            "{} {} {} {}",
            style("⊖").dim(),
            file.filename,
            format!("[{}]", file.service_type_name).dimmed(),
            style("skipped (already exists)").dimmed()
        ));
    }

    Ok(())
}

/// Load default configuration files into the database
pub fn load_default_configuration_files(ctx: &CliContext, force: bool) -> Result<()> {
    let multi = multi_progress("Default configuration files");

    let service_types = ctx.db.list_service_types()?;

    let service_type_name = |st: &ServiceType| -> String {
        service_types
            .iter()
            .find(|x| st.is(x.id))
            .unwrap()
            .name
            .clone()
    };

    // Insert Bitcoin Core configuration file template (for a miner)
    install_file(
        ctx,
        &multi,
        InstallFile {
            filename: "bitcoin.conf",
            description: "Bitcoin Core configuration file template",
            service_type: ServiceType::BitcoinMiner,
            service_type_name: service_type_name(&ServiceType::BitcoinMiner),
            destination_dir: "/opt/bitcoin/",
            default_contents: BITCOIN_CONF,
            file_type: FileType::HandlebarsTemplate,
        },
        force,
    )?;

    // Insert Bitcoin Core configuration file template (for a miner)
    install_file(
        ctx,
        &multi,
        InstallFile {
            filename: "bitcoin.conf",
            description: "Bitcoin Core configuration file template",
            service_type: ServiceType::BitcoinFollower,
            service_type_name: service_type_name(&ServiceType::BitcoinFollower),
            destination_dir: "/opt/bitcoin/",
            default_contents: BITCOIN_CONF,
            file_type: FileType::HandlebarsTemplate,
        },
        force,
    )?;

    install_file(
        ctx,
        &multi,
        InstallFile {
            filename: "stacks-node.toml",
            description: "Stacks Node configuration file template",
            service_type: ServiceType::StacksMiner,
            service_type_name: service_type_name(&ServiceType::StacksMiner),
            destination_dir: "/opt/stackify/config/",
            default_contents: STACKS_NODE_CONF,
            file_type: FileType::HandlebarsTemplate,
        },
        force,
    )?;

    install_file(
        ctx,
        &multi,
        InstallFile {
            filename: "stacks-node.toml",
            description: "Stacks Node configuration file template",
            service_type: ServiceType::StacksFollower,
            service_type_name: service_type_name(&ServiceType::StacksFollower),
            destination_dir: "/opt/stackify/config/",
            default_contents: STACKS_NODE_CONF,
            file_type: FileType::HandlebarsTemplate,
        },
        force,
    )?;

    install_file(
        ctx,
        &multi,
        InstallFile {
            filename: "stacks-signer.toml",
            description: "Stacks Signer configuration file template",
            service_type: ServiceType::StacksSigner,
            service_type_name: service_type_name(&ServiceType::StacksSigner),
            destination_dir: "/opt/stackify/config/",
            default_contents: STACKS_SIGNER_CONF,
            file_type: FileType::HandlebarsTemplate,
        },
        force,
    )?;

    multi.stop();

    Ok(())
}
