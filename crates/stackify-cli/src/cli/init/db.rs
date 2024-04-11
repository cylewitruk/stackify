use cliclack::{multi_progress, spinner, MultiProgress};
use color_eyre::Result;
use console::style;
use stackify_common::{FileType, ServiceType, ValueType};

use crate::{
    cli::{context::CliContext, log::clilog, theme::ThemedObject},
    db::{
        diesel::schema::service_type_param::allowed_values, InsertServiceFile, InsertServiceParam,
    },
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

struct AssertParam<'a> {
    name: &'a str,
    service_types: Vec<ServiceType>,
    key: &'a str,
    description: &'a str,
    default_value: Option<&'a str>,
    allowed_values: Option<Vec<&'a str>>,
    is_required: bool,
    value_type: ValueType,
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

fn assert_param(ctx: &CliContext, param: AssertParam, force: bool) -> Result<()> {
    for service_type in &param.service_types {
        if force
            || !ctx
                .db
                .check_if_service_type_param_exists(service_type.clone() as i32, &param.name)?
        {
            let allowed_vals = param.allowed_values.clone().map(|x| x.join(","));
            let param = InsertServiceParam {
                name: param.name.into(),
                service_type: service_type,
                key: param.key.into(),
                description: param.description.into(),
                default_value: param.default_value.map(|x| x.into()),
                allowed_values: allowed_vals.as_deref(),
                is_required: param.is_required,
                value_type: &param.value_type,
            };
            clilog!(
                "Inserting param: {} [{:?}], force={}",
                param.name,
                service_type,
                force
            );
            ctx.db.insert_service_param(&param)?;
        } else {
            clilog!(
                "Skipping param: {} [{:?}], force={}",
                param.name,
                param.service_types,
                force
            );
        }
    }

    Ok(())
}

pub fn load_default_configuration_params(ctx: &CliContext, force: bool) -> Result<()> {
    let spinner = spinner();
    spinner.start("Default configuration parameters");

    assert_param(
        ctx,
        AssertParam {
            name: "Bitcoin Block Frequency",
            service_types: vec![ServiceType::BitcoinMiner],
            key: "bitcoin_block_frequency",
            description:
                "The frequency at which the Bitcoin Miner will mine a new block (in seconds).",
            default_value: Some("30"),
            allowed_values: None,
            is_required: false,
            value_type: ValueType::Integer,
        },
        force,
    )?;

    assert_param(
        ctx,
        AssertParam {
            name: "PoX Sync Sample Seconds",
            service_types: vec![ServiceType::StacksMiner],
            key: "pox_sync_sample_secs",
            description: "The number of seconds to wait between PoX syncs",
            default_value: Some("5"),
            allowed_values: None,
            is_required: false,
            value_type: ValueType::Integer,
        },
        force,
    )?;

    assert_param(
        ctx,
        AssertParam {
            name: "Microblock Wait Time",
            service_types: vec![ServiceType::StacksMiner],
            key: "wait_time_for_microblocks",
            description: "The number of seconds to wait for microblocks",
            default_value: Some("0"),
            allowed_values: None,
            is_required: false,
            value_type: ValueType::Integer,
        },
        force,
    )?;

    assert_param(
        ctx,
        AssertParam {
            name: "Local Peer Seed",
            service_types: vec![ServiceType::StacksMiner, ServiceType::StacksFollower],
            key: "local_peer_seed",
            description: "The private key to use for signing P2P messages in the networking stack",
            default_value: Some("0000000000000000000000000000000000000000000000000000000000000000"),
            allowed_values: None,
            is_required: false,
            value_type: ValueType::String,
        },
        force,
    )?;

    assert_param(
        ctx,
        AssertParam {
            name: "Seed",
            service_types: vec![ServiceType::StacksMiner],
            key: "seed",
            description: "The private key to use for mining",
            default_value: Some("0000000000000000000000000000000000000000000000000000000000000000"),
            allowed_values: None,
            is_required: false,
            value_type: ValueType::String,
        },
        force,
    )?;

    assert_param(
        ctx,
        AssertParam {
            name: "Mine Microblocks",
            service_types: vec![ServiceType::StacksMiner],
            key: "mine_microblocks",
            description: "Determines whether the node will mine microblocks",
            default_value: Some("true"),
            allowed_values: None,
            is_required: false,
            value_type: ValueType::Boolean,
        },
        force,
    )?;

    spinner.stop(format!(
        "{} {}",
        style("✔").green(),
        "Default configuration parameters"
    ));

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
