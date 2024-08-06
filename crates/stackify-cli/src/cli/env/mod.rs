use color_eyre::eyre::{bail, eyre, Result};
use console::style;
use docker_api::opts::{
    ContainerCreateOpts, ContainerListOpts, ContainerStopOpts, NetworkCreateOpts,
};
use stackify_common::types::EnvironmentName;

use crate::cli::context::CliContext;
use crate::cli::theme::ThemedObject;
use crate::cli::warn;
use crate::docker::opts::{CreateContainer, CreateNetwork, ListContainers};
use crate::errors::CliError;

use self::args::EnvArgs;
use self::epoch::exec_epoch;
use self::service::exec_service;

use super::info;

pub mod args;
pub mod build;
pub mod down;
pub mod epoch;
pub mod list;
pub mod service;
pub mod start;
pub mod stop;
pub mod keychain;

pub async fn exec(ctx: &CliContext, args: EnvArgs) -> Result<()> {
    match args.commands {
        args::EnvSubCommands::List(inner_args) => list::exec(ctx, inner_args).await,
        args::EnvSubCommands::New(inner_args) => exec_create(ctx, inner_args).await,
        args::EnvSubCommands::Remove(inner_args) => exec_delete(ctx, inner_args).await,
        args::EnvSubCommands::Start(inner_args) => start::exec(ctx, inner_args).await,
        args::EnvSubCommands::Stop(inner_args) => stop::exec(ctx, inner_args).await,
        args::EnvSubCommands::Inspect(inner_args) => exec_inspect(ctx, inner_args).await,
        args::EnvSubCommands::Down(inner_args) => down::exec(ctx, inner_args).await,
        args::EnvSubCommands::Build(inner_args) => build::exec(ctx, inner_args).await,
        args::EnvSubCommands::Service(inner_args) => exec_service(ctx, inner_args).await,
        args::EnvSubCommands::Epoch(inner_args) => exec_epoch(ctx, inner_args),
        args::EnvSubCommands::Set(inner_args) => exec_set(ctx, inner_args).await,
        args::EnvSubCommands::Keychain(inner_args) => keychain::exec(ctx, inner_args).await,
    }
}

async fn exec_set(_ctx: &CliContext, args: args::SetArgs) -> Result<()> {
    println!("Set environment: {}", args.env_name);
    Ok(())
}

async fn exec_inspect(_ctx: &CliContext, args: args::InspectArgs) -> Result<()> {
    println!("Inspect environment: {}", args.env_name);
    Ok(())
}

async fn exec_create(ctx: &CliContext, args: args::NewArgs) -> Result<()> {
    let env_name = EnvironmentName::new(&args.env_name)?;
    let env = ctx
        .db
        .create_environment(env_name.as_ref(), args.bitcoin_block_speed)?;
    println!("Environment created: {}", env.id);
    Ok(())
}

async fn exec_delete(_ctx: &CliContext, _args: args::RemoveArgs) -> Result<()> {
    println!("Delete environment");
    Ok(())
}

/// Prompts the user to select an environment from the list of available environments.
/// This function is used when the user does not provide an environment name as an argument
/// and is required to select an environment interactively.
pub fn prompt_environment_name(ctx: &CliContext) -> Result<EnvironmentName> {
    let environments = ctx.db.list_environments()?;

    if environments.is_empty() {
        bail!(CliError::Graceful { 
            title: "No environments are configured".to_string(), 
            message: format!(
                "Please create a new environment using the `{}` command first.", 
                "stackify env new".bold()
            )
        });
    }

    let env_items = environments
        .iter()
        .map(|env| (env.name.as_str(), env.name.as_str(), ""))
        .collect::<Vec<_>>();

    let env_name = cliclack::select("Select an environment")
        .items(&env_items)
        .interact()?;

    Ok(EnvironmentName::new(env_name)?)
}