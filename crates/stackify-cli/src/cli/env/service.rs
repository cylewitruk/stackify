use color_eyre::eyre::{bail, eyre};
use color_eyre::Result;
use inquire::{Confirm, Select, Text};
use stackify_common::EnvironmentName;

use crate::cli::theme::ThemedObject;
use crate::cli::{info, warn};
use crate::db::model::Epoch;
use crate::util::FilterByServiceType;

use super::CliContext;

use super::args::{
    ServiceAddArgs, ServiceArgs, ServiceInspectArgs, ServiceListArgs, ServiceRemoveArgs,
    ServiceSubCommands,
};

pub fn exec_service(ctx: &CliContext, args: ServiceArgs) -> Result<()> {
    match args.commands {
        ServiceSubCommands::Add(inner_args) => {
            exec_add(ctx, inner_args)?;
        }
        ServiceSubCommands::Remove(inner_args) => {
            exec_remove(ctx, inner_args)?;
        }
        ServiceSubCommands::Inspect(inner_args) => {
            exec_inspect(ctx, inner_args)?;
        }
        ServiceSubCommands::List(inner_args) => {
            exec_list(ctx, inner_args)?;
        }
        ServiceSubCommands::Config => {
            exec_config(ctx)?;
        }
    }
    Ok(())
}

fn exec_add(ctx: &CliContext, args: ServiceAddArgs) -> Result<()> {
    let _env_name = EnvironmentName::new(&args.env_name)?;

    let service_types = ctx.db.list_service_types()?;

    // Collect service type
    let service_type_names = service_types.iter().map(|st| st.name.clone()).collect::<Vec<_>>();
    let service_type = Select::new("Select a service type", service_type_names)
        .prompt()?;
    let service_type = service_types.iter().find(|st| st.name == service_type)
        .ok_or(eyre!("Service type not found"))?;
    
    // Collect service version
    let all_service_versions = ctx.db.list_service_versions()?;
    let service_versions = all_service_versions
        .filter_by_service_type(service_type.id);
    let service_version_names = service_versions.iter().map(|sv| sv.version.clone()).collect::<Vec<_>>();
    let service_version = Select::new("Select a service version", service_version_names)
        .prompt()?;
    let service_version = service_versions.iter().find(|sv| sv.version == service_version)
        .ok_or(eyre!("Service version not found"))?;

    // When to start?
    let start_opts = vec![
        "At a specific block height",
        "At a specific epoch",
        "I'll do this later"
    ];
    let start_type = Select::new( 
        &format!("Choose when {} {} should start:", service_type.name.cyan(), service_version.version.cyan()),
        start_opts.clone()
    ).prompt()?;

    let epochs = ctx.db.list_epochs()?;
        let epoch_names = epochs.iter().map(|e| e.name.clone()).collect::<Vec<_>>();
        let mut start_epoch: Option<String> = None;
        let mut start_height: Option<u32> = None;

    if start_type == start_opts[2] {
        warn("Note that if you do not set a start point, the service will not be started automatically when the environment runs.");
    } else if start_type == start_opts[0] {
        start_height = Some(inquire::prompt_u32("Enter the block height at which the service should start:")?);
    } else if start_type == start_opts[1] {
        let start_epoch_str = Select::new("Select an epoch at which the service should start:", epoch_names.clone())
            .prompt()?;
        start_epoch = Some(ctx.db.list_epochs()?.iter()
            .find(|e| e.name == start_epoch_str)
            .ok_or(eyre!("Epoch not found"))?
            .clone().name);
    }

    // When to stop?
    let stop_opts = vec![
        "At a specific block height",
        "At a specific epoch", 
        "The service should run indefinitely (default)"
    ];

    let stop_type = Select::new(
        &format!("Choose when {} {} should stop:", service_type.name.cyan(), service_version.version.cyan()),
        stop_opts.clone()
    ).prompt()?;

    let mut stop_epoch: Option<Epoch> = None;
    let mut stop_height: Option<u32> = None;

    if stop_type == stop_opts[2] {
        info("Note that if you do not set a stop point, the service will not be stopped automatically when the environment runs.");
    } else if stop_type == stop_opts[0] {
        stop_height = Some(inquire::prompt_u32("Enter the block height at which the service should stop:")?);
    } else if stop_type == stop_opts[1] {
        let stop_epoch_str = Select::new("Select an epoch at which the service should stop:", epoch_names.clone())
            .prompt()?;
        stop_epoch = Some(ctx.db.list_epochs()?
            .iter()
            .find(|e| e.name == stop_epoch_str)
            .ok_or(eyre!("Epoch not found"))?
            .clone());
    }

    todo!()
}

fn exec_remove(_ctx: &CliContext, args: ServiceRemoveArgs) -> Result<()> {
    let _env_name = EnvironmentName::new(&args.env_name)?;
    todo!()
}

fn exec_inspect(_ctx: &CliContext, args: ServiceInspectArgs) -> Result<()> {
    let _env_name = EnvironmentName::new(&args.env_name)?;
    todo!()
}

fn exec_list(_ctx: &CliContext, args: ServiceListArgs) -> Result<()> {
    let _env_name = EnvironmentName::new(&args.env_name)?;
    todo!()
}

fn exec_config(_ctx: &CliContext) -> Result<()> {
    todo!()
}