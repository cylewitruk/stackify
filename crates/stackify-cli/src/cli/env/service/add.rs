use clap::Args;
use color_eyre::{eyre::bail, Result};
use stackify_common::{types::EnvironmentName, util::random_hex, ServiceAction, ServiceType};

use crate::{
    cli::{context::CliContext, log::clilog, theme::ThemedObject},
    db::{cli_db::CliDatabase, diesel::model::Epoch},
    errors::CliError,
    util::FilterByServiceType,
};

#[derive(Debug, Args)]
pub struct ServiceAddArgs {
    /// Indicates whether or not an interactive prompt should be used for providing
    /// the required information for this command (recommended!). This flag is
    /// set by default.
    #[arg(required = false, short = 'i', default_value = "true")]
    pub interactive: bool,

    /// The name of the environment to which the service should be added.
    #[arg(required = true, value_name = "ENV_NAME")]
    pub env_name: String,
}

pub async fn exec(ctx: &CliContext, args: ServiceAddArgs) -> Result<()> {
    let env_name = EnvironmentName::new(&args.env_name)?;
    let env = ctx.db.load_environment(env_name.as_ref())?;
    let service_types = ctx.db.list_service_types()?;
    let epochs = ctx.db.list_epochs()?;

    cliclack::intro("Add new environment service".bold())?;

    cliclack::log::remark(format!(
        "You are about to add a new service to the environment '{}'.",
        env_name.magenta().bold()
    ))?;

    // Collect service type
    let service_type = cliclack::select("Which type of service would you like to add?")
        .items(
            &service_types
                .iter()
                .map(|st| (st.clone(), &st.name, ""))
                .collect::<Vec<_>>(),
        )
        .interact()?;

    if [
        ServiceType::StacksMiner,
        ServiceType::StacksFollower,
        ServiceType::StacksSigner,
        ServiceType::StacksTransactionGenerator,
    ]
    .contains(&ServiceType::from_i32(service_type.id)?)
        && env.keychains.is_empty()
    {
        cliclack::log::error("This service type requires a Stacks keychain, but no keychains have been added to this environment.")?;
        cliclack::outro_note(
            "No keychains available".red().bold(),
            format!(
                "Please add a keychain to the environment first using the command '{}'.",
                format!("stackify env keychain new {}", &env_name).cyan()
            ),
        )?;
        return Ok(());
    }

    // Collect service version
    let all_service_versions = ctx.db.list_service_versions()?;
    let service_versions = all_service_versions.filter_by_service_type(service_type.id);
    let service_version = cliclack::select("Which version?")
        .items(
            &service_versions
                .into_iter()
                .map(|sv| (sv.clone(), &sv.version, ""))
                .collect::<Vec<_>>(),
        )
        .interact()?;

    // When should the service be started?
    let start_at = cliclack::select("When should this service start?")
        .item(StartAtKind::Immediate, "Immediately", "default: block 0")
        .item(StartAtKind::BlockHeight, "At a specific block height", "")
        .item(StartAtKind::Epoch, "At a specific epoch", "")
        .item(StartAtKind::Never, "I'll do this later", "")
        .interact()?;

    let start_at = match start_at {
        StartAtKind::Epoch => {
            let epoch = cliclack::select("Which epoch should the service start at?")
                .items(
                    &epochs
                        .iter()
                        .map(|e| (e.clone(), &e.name, ""))
                        .collect::<Vec<_>>(),
                )
                .interact()?;
            StartAt::Epoch(epoch)
        }
        StartAtKind::BlockHeight => {
            let block_height = cliclack::input("What block height should the service start at?")
                .validate(|input: &String| match input.parse::<u32>() {
                    Ok(_) => Ok(()),
                    Err(_) => {
                        Err("Invalid block height. Please enter a valid positive number (>= 0).")
                    }
                })
                .interact()?;
            StartAt::BlockHeight(block_height)
        }
        StartAtKind::Immediate => StartAt::BlockHeight(0),
        StartAtKind::Never => StartAt::Never,
    };

    let stop_at = if start_at == StartAt::Never {
        StopAt::Never
    } else {
        let stop_at = cliclack::select("When should this service stop?")
            .item(StopAtKind::Never, "Never", "default")
            .item(StopAtKind::BlockHeight, "At a specific block height", "")
            .item(StopAtKind::Epoch, "At a specific epoch", "")
            .interact()?;

        match stop_at {
            StopAtKind::BlockHeight => {
                let block_height = cliclack::input("What block height should the service stop at?")
                    .validate(|input: &String| match input.parse::<u32>() {
                        Ok(_) => Ok(()),
                        Err(_) => Err(
                            "Invalid block height. Please enter a valid positive number (>= 0).",
                        ),
                    })
                    .interact()?;
                StopAt::BlockHeight(block_height)
            }
            StopAtKind::Epoch => {
                let epoch = cliclack::select("Which epoch should the service stop at?")
                    .items(
                        &epochs
                            .iter()
                            .map(|e| (e.clone(), &e.name, ""))
                            .collect::<Vec<_>>(),
                    )
                    .interact()?;
                StopAt::Epoch(epoch)
            }
            _ => StopAt::Never,
        }
    };

    random_hex(4);
    let name = format!(
        "{}-{}-{}",
        env_name.to_string(),
        service_type.cli_name,
        random_hex(4)
    );

    // If the service requires a Stacks keychain, have the user select one.
    // If no keychains are added, the user will be prompted to add one first.
    let stacks_keychain = if [
        ServiceType::StacksMiner,
        ServiceType::StacksFollower,
        ServiceType::StacksSigner,
        ServiceType::StacksTransactionGenerator,
    ]
    .contains(&ServiceType::from_i32(service_type.id)?)
    {
        let keychains = env
            .keychains
            .iter()
            .map(|kc| {
                (
                    &kc.stx_address,
                    &kc.stx_address,
                    kc.remark.clone().unwrap_or_default(),
                )
            })
            .collect::<Vec<_>>();

        if keychains.is_empty() {
            cliclack::log::error(
                "No keychains found for this environment. Please add a keychain first.",
            )?;
            bail!(CliError::Graceful {
                title: "No keychains found".to_string(),
                message: "No keychains found for this environment. Please add a keychain first."
                    .to_string()
            });
        }

        let stx_address = cliclack::select("Select a keychain for this service")
            .items(&keychains)
            .interact()?;

        let keychain = ctx
            .db
            .get_environment_keychain_by_stx_address(&stx_address)?;

        if let Some(keychain) = keychain {
            Some(keychain.stx_address)
        } else {
            bail!(CliError::Graceful {
                title: "Keychain not found".to_string(),
                message: "The selected keychain was not found. This is a bug.".to_string()
            });
        }
    } else {
        None
    };

    let comment: String = cliclack::input("Comment:")
        .placeholder("Write a short comment about this service")
        .required(false)
        .interact()?;

    cliclack::log::success(format!(
        "{}\n{}",
        "Configuration complete!".green().bold(),
        "Please review the above and confirm the addition of the service to the environment."
    ))?;

    let stacks_node = if ServiceType::StacksSigner.is(service_type.id) {
        let stacks_peers = env
            .services
            .iter()
            .filter(|service| {
                [ServiceType::StacksMiner, ServiceType::StacksFollower]
                    .contains(&ServiceType::from_i32(service.service_type.id).unwrap())
            })
            .filter(|svc| &svc.name != &name)
            .map(|service| service.name.clone())
            .collect::<Vec<_>>();

        let stacks_node =
            cliclack::select("Which Stacks node should this signer receive events from?")
                .items(
                    &stacks_peers
                        .iter()
                        .map(|sn| (sn.clone(), sn, ""))
                        .collect::<Vec<_>>(),
                )
                .interact()?;

        Some(stacks_node)
    } else {
        None
    };

    let add = cliclack::confirm("Add the above service to the environment?").interact()?;

    if !add {
        cliclack::outro_cancel("Aborted by user".red().bold())?;
        return Ok(());
    }

    // Add the service
    let env_service = ctx.db.add_environment_service(
        env.id,
        service_version.id,
        &name,
        if comment.is_empty() {
            None
        } else {
            Some(&comment)
        },
    )?;

    // If this is a service which requires a Stacks keychain, the keychain variable
    // will be populated with the Stacks keychain address. Add this to the service.
    if let Some(keychain) = stacks_keychain {
        clilog!("Adding keychain to service: {:?}", keychain);
        let param_id = ctx
            .db
            .find_service_type_param_id_by_key(service_type.id, "stacks_keychain")?;
        ctx.db
            .add_environment_service_param(env_service.id, param_id, &keychain)?;
    }

    // If this is a service which requires a Stacks node, the stacks_node variable
    // will be populated with the Stacks node name. Add this to the service.
    if let Some(stacks_node) = stacks_node {
        let param_id = ctx
            .db
            .find_service_type_param_id_by_key(service_type.id, "stacks_node")?;

        ctx.db
            .add_environment_service_param(env_service.id, param_id, &stacks_node)?;
    }

    // Add start actions to the service depending on what the user selected
    if let StartAt::BlockHeight(block_height) = start_at {
        ctx.db.add_environment_service_action(
            env_service.id,
            ServiceAction::StartService as i32,
            Some(block_height as i32),
            None,
        )?;
    } else if let StartAt::Epoch(epoch) = start_at {
        ctx.db.add_environment_service_action(
            env_service.id,
            ServiceAction::StartService as i32,
            None,
            Some(epoch.id),
        )?;
    }

    // for cfg_file in ctx
    //     .db
    //     .list_service_files_for_service_type(service_type.id)?
    // {
    //     ctx.db.add_environment_service_file(
    //         env.id,
    //         env_service.id,
    //         cfg_file.id,
    //         &cfg_file.default_contents,
    //     )?;
    // }

    //cliclack::log::success("Service added successfully!")?;
    cliclack::outro(format!(
        "The service {} has been added to the environment {}.",
        name.bold(),
        env_name.bold()
    ))?;

    Ok(())
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum StartAtKind {
    Immediate,
    BlockHeight,
    Epoch,
    Never,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum StartAt {
    BlockHeight(u32),
    Epoch(Epoch),
    Never,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum StopAtKind {
    BlockHeight,
    Epoch,
    Never,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum StopAt {
    BlockHeight(u32),
    Epoch(Epoch),
    Never,
}
