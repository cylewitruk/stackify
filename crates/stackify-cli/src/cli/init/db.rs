use color_eyre::Result;
use console::style;
use stackify_common::{FileType, ServiceType};

use crate::{
    cli::context::CliContext,
    db::InsertServiceFile,
    includes::{BITCOIN_CONF, STACKS_NODE_CONF, STACKS_SIGNER_CONF},
};

/// TODO: Super ugly... just doing this to get it done.
pub fn load_default_configuration_files(ctx: &CliContext) -> Result<()> {
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
            default_contents: BITCOIN_CONF.to_vec(),
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
            default_contents: BITCOIN_CONF.to_vec(),
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
            default_contents: STACKS_NODE_CONF.to_vec(),
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
            default_contents: STACKS_NODE_CONF.to_vec(),
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
            default_contents: STACKS_SIGNER_CONF.to_vec(),
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
