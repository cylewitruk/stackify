use clap::Args;
use color_eyre::Result;
use stackify_common::{types::EnvironmentName, util::random_hex, ServiceAction};

use crate::{
    cli::{context::CliContext, theme::ThemedObject},
    db::diesel::model::Epoch,
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
    #[arg(required = true, value_name = "NAME", short = 'e', long = "env")]
    pub env_name: String,
}

pub fn exec(ctx: &CliContext, args: ServiceAddArgs) -> Result<()> {
    let env_name = EnvironmentName::new(&args.env_name)?;
    let env = ctx.db.get_environment_by_name(env_name.as_ref())?;
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
                    Ok(i) => Ok(()),
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
                        Ok(i) => Ok(()),
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

    let comment: String = cliclack::input("Comment:")
        .placeholder("Write a short comment about this service")
        .required(false)
        .interact()?;

    let comment = if comment.is_empty() {
        None
    } else {
        Some(comment)
    };

    random_hex(4);
    let name = format!(
        "{}-{}-{}",
        env_name.to_string(),
        service_type.cli_name,
        random_hex(4)
    );

    cliclack::log::success(format!(
        "{}\n{}",
        "Configuration complete!".green().bold(),
        "Please review the above and confirm the addition of the service to the environment.
    "
    ))?;

    let add = cliclack::confirm("Add the above service to the environment?").interact()?;

    if !add {
        cliclack::outro_cancel("Aborted by user".red().bold())?;
        return Ok(());
    }

    // Add the service
    let env_service =
        ctx.db
            .add_environment_service(env.id, service_version.id, &name, comment.as_deref())?;

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

    for cfg_file in ctx
        .db
        .list_service_files_for_service_type(service_type.id)?
    {
        ctx.db.add_environment_service_file(
            env.id,
            env_service.id,
            cfg_file.id,
            &cfg_file.default_contents,
        )?;
    }

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
