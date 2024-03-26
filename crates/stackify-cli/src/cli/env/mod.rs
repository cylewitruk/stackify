use color_eyre::eyre::{eyre, Result};
use comfy_table::{Cell, Color, ColumnConstraint, Table, Width};
use console::style;
use stackify_common::{docker::ListStackifyContainerOpts, EnvironmentName};

use crate::cli::{warn, PAD_WIDTH};
use crate::cli::context::CliContext;
use crate::util::print::{print_fail, print_ok};
use crate::util::progressbar::PbWrapper;

use self::args::EnvArgs;
use self::epoch::exec_epoch;
use self::service::exec_service;

use super::{error, info};

pub mod args;
pub mod build;
pub mod service;
pub mod epoch;

pub fn exec(ctx: &CliContext, args: EnvArgs) -> Result<()> {
    match args.commands {
        args::EnvSubCommands::List(inner_args) => exec_list(ctx, inner_args),
        args::EnvSubCommands::Create(inner_args) => exec_create(ctx, inner_args),
        args::EnvSubCommands::Delete(inner_args) => exec_delete(ctx, inner_args),
        args::EnvSubCommands::Start(inner_args) => exec_start(ctx, inner_args),
        args::EnvSubCommands::Stop(inner_args) => exec_stop(ctx, inner_args),
        args::EnvSubCommands::Inspect(inner_args) => exec_inspect(ctx, inner_args),
        args::EnvSubCommands::Down(inner_args) => exec_down(ctx, inner_args),
        args::EnvSubCommands::Build(inner_args) => build::exec(ctx, inner_args),
        args::EnvSubCommands::Service(inner_args) => exec_service(ctx, inner_args),
        args::EnvSubCommands::Epoch(inner_args) => exec_epoch(ctx, inner_args),
        args::EnvSubCommands::Set(inner_args) => exec_set(ctx, inner_args),
    }
}

fn exec_set(_ctx: &CliContext, args: args::SetArgs) -> Result<()> {
    println!("Set environment: {}", args.env_name);
    Ok(())
}

fn exec_inspect(_ctx: &CliContext, args: args::InspectArgs) -> Result<()> {
    println!("Inspect environment: {}", args.env_name);
    Ok(())
}

fn exec_list(ctx: &CliContext, _args: args::ListArgs) -> Result<()> {
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

fn exec_create(ctx: &CliContext, args: args::CreateArgs) -> Result<()> {
    let env_name = EnvironmentName::new(&args.env_name)?;
    let env = ctx
        .db
        .create_environment(env_name.as_ref(), args.bitcoin_block_speed)?;
    println!("Environment created: {}", env.id);
    Ok(())
}

fn exec_delete(_ctx: &CliContext, _args: args::DeleteArgs) -> Result<()> {
    println!("Delete environment");
    Ok(())
}

fn exec_start(ctx: &CliContext, args: args::StartArgs) -> Result<()> {
    let env_name = EnvironmentName::new(&args.env_name)?;

    // Check if the environment has any services defined. If not, return an error.
    let env = ctx.db.list_environment_services(env_name.as_ref())?;
    if env.is_empty() {
        warn(format!(
            "The '{}' environment has no services defined, so there is nothing to start.\n",
            env_name)
        );
        println!("Please define at least one service before starting the environment.");
        println!(
            "See the {} command for more information.",
            style("stackify env service").white().bold()
        );
        return Ok(());
    }

    // Assert that the environment is not already running.
    let existing_containers = ctx
        .docker
        .list_stackify_containers(ListStackifyContainerOpts {
            environment_name: Some(env_name.clone()),
            only_running: Some(true),
        })?;

    // If there are any running containers, we can't start the environment.
    if !existing_containers.is_empty() {
        error("The environment is already running.");
    }

    // Create the environment container. This is our "lock file" for the environment
    // within Docker -- it's the first resource we create and the last one we delete.
    PbWrapper::new_spinner("Create environment container").exec(|pb| {
        let env_container = ctx.docker.create_environment_container(&env_name)?;
        pb.finish_success_with_meta(&env_container.id);
        if !env_container.warnings.is_empty() {
            warn("Warnings were generated while creating the environment container:");
            env_container
                .warnings
                .iter()
                .for_each(|w| println!("  - {}", w));
        }
        Ok(())
    })?;

    // Create the network for the environment.
    PbWrapper::new_spinner("Creating environment network").exec(|pb| {
        let network = ctx.docker.create_stackify_network(&env_name)?;
        pb.finish_success_with_meta(&network.id);
        Ok(())
    })?;

    Ok(())
}

fn exec_stop(ctx: &CliContext, args: args::StopArgs) -> Result<()> {
    let env_name = EnvironmentName::new(&args.env_name)?;

    let containers = ctx
        .docker
        .list_stackify_containers(ListStackifyContainerOpts {
            environment_name: Some(env_name.clone()),
            only_running: Some(true),
        })?;

    if containers.is_empty() {
        info("There are no running containers for the environment.\n");
        println!(
            "To start the environment, use the {} command.",
            style("stackify env start").white().bold()
        );
        return Ok(());
    }

    for container in containers {
        print!("Stopping container: {:PAD_WIDTH$}", container.name);
        match ctx.docker.stop_container(&container.id) {
            Ok(_) => print_ok(None),
            Err(_) => print_fail(None),
        }
    }

    Ok(())
}

fn exec_down(ctx: &CliContext, args: args::DownArgs) -> Result<()> {
    let env_name = EnvironmentName::new(&args.env_name)?;

    let containers = ctx
        .docker
        .list_stackify_containers(ListStackifyContainerOpts {
            environment_name: Some(env_name.clone()),
            only_running: Some(false),
        })?;

    if containers.is_empty() {
        info("There are no built containers for this environment.\n");
        println!(
            "To build the environment, use the {} command.",
            style("stackify env build").white().bold()
        );
        return Ok(());
    }

    for container in containers {
        PbWrapper::new_spinner(format!("Removing container: {}", container.name))
            .exec(|_| ctx.docker.rm_container(&container.id))?;
    }

    Ok(())
}
