use color_eyre::eyre::{eyre, Result};
use comfy_table::{Cell, Color, ColumnConstraint, Table, Width};
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
pub mod service;

pub async fn exec(ctx: &CliContext, args: EnvArgs) -> Result<()> {
    match args.commands {
        args::EnvSubCommands::List(inner_args) => exec_list(ctx, inner_args).await,
        args::EnvSubCommands::Create(inner_args) => exec_create(ctx, inner_args).await,
        args::EnvSubCommands::Delete(inner_args) => exec_delete(ctx, inner_args).await,
        args::EnvSubCommands::Start(inner_args) => exec_start(ctx, inner_args).await,
        args::EnvSubCommands::Stop(inner_args) => exec_stop(ctx, inner_args).await,
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

async fn exec_list(ctx: &CliContext, _args: args::ListArgs) -> Result<()> {
    let environments = ctx.db.list_environments()?;

    if environments.is_empty() {
        info("No environments found.\n");
        println!(
            "To create a new environment, use the {} command.",
            style("stackify env create").white().bold()
        );
        return Ok(());
    }

    let mut table = Table::new();
    table
        .set_header(vec![
            Cell::new("NAME").fg(Color::Cyan),
            Cell::new("CREATED").fg(Color::Cyan),
            Cell::new("LAST RUN").fg(Color::Cyan),
        ])
        .load_preset(comfy_table::presets::NOTHING);

    table
        .column_mut(0)
        .ok_or(eyre!("Failed to retrieve column."))?
        .set_constraint(ColumnConstraint::LowerBoundary(Width::Fixed(40)));

    for env in environments {
        table.add_row(vec![
            env.name,
            env.created_at.date().to_string(),
            env.updated_at.date().to_string(),
        ]);
    }

    println!("{table}");
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

async fn exec_start(ctx: &CliContext, args: args::StartArgs) -> Result<()> {
    let env_name = EnvironmentName::new(&args.env_name)?;
    let env = ctx.db.get_environment_by_name(env_name.as_ref())?;

    // Check if the environment has any services defined. If not, return an error.
    let env_services = ctx
        .db
        .list_environment_services_for_environment_id(env.id)?;
    if env_services.is_empty() {
        warn(format!(
            "The '{}' environment has no services defined, so there is nothing to start.\n",
            env_name
        ));
        println!("Please define at least one service before starting the environment.");
        println!(
            "See the {} command for more information.",
            style("stackify env service").white().bold()
        );
        return Ok(());
    }

    // Assert that the environment is not already running.
    let existing_containers = ctx
        .docker()
        .api()
        .containers()
        .list(&ContainerListOpts::for_all_stackify_containers())
        .await?;

    // If there are any running containers, we can't start the environment.
    if !existing_containers.is_empty() {
        cliclack::log::error("The environment is already running.");
    }

    // Create the environment container. This is our "lock file" for the environment
    // within Docker -- it's the first resource we create and the last one we delete.
    let mut spinner = cliclack::spinner();
    spinner.start("Creating environment container...");
    ctx.docker()
        .api()
        .containers()
        .create(&ContainerCreateOpts::for_stackify_environment_container(
            &env_name,
        ))
        .await?;
    spinner.stop("Environment container created");

    // Create the network for the environment.
    let mut spinner = cliclack::spinner();
    spinner.start("Creating environment network...");
    ctx.docker()
        .api()
        .networks()
        .create(&NetworkCreateOpts::for_stackify_environment(&env_name))
        .await?;
    spinner.stop("Environment network created");

    Ok(())
}

async fn exec_stop(ctx: &CliContext, args: args::StopArgs) -> Result<()> {
    let env_name = EnvironmentName::new(&args.env_name)?;

    let containers = ctx
        .docker()
        .api()
        .containers()
        .list(&ContainerListOpts::running_in_environment(&env_name))
        .await?;

    if containers.is_empty() {
        info("There are no running containers for the environment.\n");
        println!(
            "To start the environment, use the {} command.",
            style("stackify env start").white().bold()
        );
        return Ok(());
    }

    for container in containers {
        let mut spinner = cliclack::spinner();
        let container_id = container.id.ok_or(eyre!("Container ID not found."))?;
        let container_names = container.names.ok_or(eyre!("Container name not found."))?;
        let container_name = container_names.join(", ");
        spinner.start(format!("Stopping container: {}", container_name));
        ctx.docker()
            .api()
            .containers()
            .get(container_id)
            .stop(&ContainerStopOpts::default())
            .await?;
        spinner.stop(format!("Container {} stopped", container_name));
    }

    Ok(())
}

async fn exec_down(ctx: &CliContext, args: args::DownArgs) -> Result<()> {
    let env_name = EnvironmentName::new(&args.env_name)?;

    let containers = ctx
        .docker()
        .api()
        .containers()
        .list(&ContainerListOpts::for_environment(&env_name))
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
        let mut spinner = cliclack::spinner();
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
