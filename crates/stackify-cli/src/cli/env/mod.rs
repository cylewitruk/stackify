use color_eyre::eyre::{eyre, Result};
use comfy_table::{Cell, Color, ColumnConstraint, Table, Width};
use stackify_common::{docker::ListStackifyContainerOpts, EnvironmentName};

use crate::context::CliContext;
use crate::cli::PAD_WIDTH;
use crate::util::print::{print_fail, print_ok};

use self::args::EnvArgs;

pub mod args;

pub fn exec(ctx: &CliContext, args: EnvArgs) -> Result<()> {
    match args.commands {
        args::EnvSubCommands::List(inner_args) => exec_list(ctx, inner_args),
        args::EnvSubCommands::Create(inner_args) => exec_create(ctx, inner_args),
        args::EnvSubCommands::Delete(inner_args) => exec_delete(ctx, inner_args),
        args::EnvSubCommands::Start(inner_args) => exec_start(ctx, inner_args),
        args::EnvSubCommands::Stop(inner_args) => exec_stop(ctx, inner_args),
        args::EnvSubCommands::Inspect(inner_args) => exec_inspect(ctx, inner_args),
        args::EnvSubCommands::Down(inner_args) => exec_down(ctx, inner_args),
    }
}

fn exec_down(_ctx: &CliContext, _args: args::DownArgs) -> Result<()> {
    println!("Down environment");
    Ok(())
}

fn exec_inspect(_ctx: &CliContext, args: args::InspectArgs) -> Result<()> {
    println!("Inspect environment: {}", args.env_name);
    Ok(())
}

fn exec_list(ctx: &CliContext, _args: args::ListArgs) -> Result<()> {
    let environments = ctx.db.list_environments()?;

    let mut table = Table::new();
    table
        .set_header(vec![
            Cell::new("NAME").fg(Color::Cyan),
            Cell::new("CREATED").fg(Color::Cyan),
            Cell::new("LAST RUN").fg(Color::Cyan)
        ])
        .load_preset(comfy_table::presets::NOTHING);

    table.column_mut(0)
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
    let env = ctx.db.create_environment(&args.env_name.unwrap(), args.bitcoin_block_speed)?;
    println!("Environment created: {}", env.id);
    Ok(())
}

fn exec_delete(_ctx: &CliContext, _args: args::DeleteArgs) -> Result<()> {
    println!("Delete environment");
    Ok(())
}

fn exec_start(ctx: &CliContext, args: args::StartArgs) -> Result<()> {
    println!("Start environment: {}", args.env_name);
    let env_name = EnvironmentName::new(&args.env_name)?;

    // Assert that the environment is not already running.
    let existing_containers = ctx.docker.list_stackify_containers(ListStackifyContainerOpts{ 
        environment_name: Some(env_name.clone()),
        running: Some(true)
    })?;

    // If there are any running containers, we can't start the environment.
    if !existing_containers.is_empty() {
        return Err(eyre!("Environment already started"));
    }
    
    // Create the environment container. This is our "lock file" for the environment
    // within Docker -- it's the first resource we create and the last one we delete.
    let env_container = ctx.docker.create_environment_container(&env_name)?;
    println!("Environment container created: {}", env_container.id);
    println!("Warnings: {:?}", env_container.warnings);

    // Create the network for the environment.
    let network = ctx.docker.create_stackify_network(&env_name)?;
    println!("Environment network created: {}", network.id);

    Ok(())
}

fn exec_stop(ctx: &CliContext, args: args::StopArgs) -> Result<()> {
    println!("Stop environment");
    let env_name = EnvironmentName::new(&args.env_name)?;

    let containers = ctx.docker.list_stackify_containers(ListStackifyContainerOpts {
        environment_name: Some(env_name.clone()),
        running: Some(true)
    })?;
    
    if containers.is_empty() {
        println!("There are no running containers for the environment.");
    }

    for container in containers {
        print!("Stopping container: {:PAD_WIDTH$}", container.name);
        match ctx.docker.stop_container(&container.id) {
            Ok(_) => print_ok(None),
            Err(e) => print_fail(None)
        }
    }

    Ok(())
}