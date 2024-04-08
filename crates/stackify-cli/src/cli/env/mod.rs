use color_eyre::eyre::{eyre, Result};
use console::style;
use docker_api::opts::{
    ContainerCreateOpts, ContainerListOpts, ContainerStopOpts, NetworkCreateOpts,
};
use stackify_common::types::EnvironmentName;

use crate::cli::context::CliContext;
use crate::cli::warn;
use crate::docker::opts::{CreateContainer, CreateNetwork, ListContainers};

use self::args::EnvArgs;
use self::epoch::exec_epoch;
use self::service::exec_service;

use super::info;

pub mod args;
pub mod build;
pub mod epoch;
pub mod list;
pub mod service;
pub mod start;
pub mod stop;

pub async fn exec(ctx: &CliContext, args: EnvArgs) -> Result<()> {
    match args.commands {
        args::EnvSubCommands::List(inner_args) => list::exec(ctx, inner_args).await,
        args::EnvSubCommands::Create(inner_args) => exec_create(ctx, inner_args).await,
        args::EnvSubCommands::Delete(inner_args) => exec_delete(ctx, inner_args).await,
        args::EnvSubCommands::Start(inner_args) => start::exec(ctx, inner_args).await,
        args::EnvSubCommands::Stop(inner_args) => stop::exec(ctx, inner_args).await,
        args::EnvSubCommands::Inspect(inner_args) => exec_inspect(ctx, inner_args).await,
        args::EnvSubCommands::Down(inner_args) => exec_down(ctx, inner_args).await,
        args::EnvSubCommands::Build(inner_args) => build::exec(ctx, inner_args).await,
        args::EnvSubCommands::Service(inner_args) => exec_service(ctx, inner_args),
        args::EnvSubCommands::Epoch(inner_args) => exec_epoch(ctx, inner_args),
        args::EnvSubCommands::Set(inner_args) => exec_set(ctx, inner_args).await,
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

async fn exec_create(ctx: &CliContext, args: args::CreateArgs) -> Result<()> {
    let env_name = EnvironmentName::new(&args.env_name)?;
    let env = ctx
        .db
        .create_environment(env_name.as_ref(), args.bitcoin_block_speed)?;
    println!("Environment created: {}", env.id);
    Ok(())
}

async fn exec_delete(_ctx: &CliContext, _args: args::DeleteArgs) -> Result<()> {
    println!("Delete environment");
    Ok(())
}

async fn exec_down(ctx: &CliContext, args: args::DownArgs) -> Result<()> {
    let env_name = EnvironmentName::new(&args.env_name)?;

    let containers = ctx
        .docker()
        .api()
        .containers()
        .list(&ContainerListOpts::for_environment(&env_name, true))
        .await?;

    if containers.is_empty() {
        info("There are no built containers for this environment.\n");
        println!(
            "To build the environment, use the {} command.",
            style("stackify env build").white().bold()
        );
        return Ok(());
    }

    for container in containers {
        let spinner = cliclack::spinner();
        let container_id = container.id.ok_or(eyre!("Container ID not found."))?;
        let container_names = container.names.ok_or(eyre!("Container name not found."))?;
        let container_name = container_names.join(", ");
        spinner.start(format!("Removing container: {}", container_name));
        ctx.docker()
            .api()
            .containers()
            .get(container_id)
            .delete()
            .await?;
        spinner.stop(format!("Container {} removed", container_name));
    }

    Ok(())
}
