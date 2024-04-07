use cliclack::{intro, outro_note};
use color_eyre::Result;
use console::style;
use docker_api::opts::{ContainerCreateOpts, ContainerListOpts, NetworkCreateOpts};
use stackify_common::types::EnvironmentName;

use crate::{
    cli::{context::CliContext, theme::ThemedObject, warn},
    docker::opts::{CreateContainer, CreateNetwork, ListContainers},
};

use super::args::StartArgs;

pub async fn exec(ctx: &CliContext, args: StartArgs) -> Result<()> {
    let env_name = EnvironmentName::new(&args.env_name)?;
    let env = ctx.db.get_environment_by_name(env_name.as_ref())?;

    intro("Start Environment".bold())?;

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
        .list(&ContainerListOpts::for_environment(&env_name, true))
        .await?;

    // If there are any running containers, we can't start the environment.
    if !existing_containers.is_empty() {
        cliclack::log::error("The environment is already running.")?;
        outro_note(
            "Environment Running".bold().red(),
            format!(
                "{} {} {}",
                style("To stop the environment, use the"),
                "stackify env stop".bold().white(),
                style("command.").dimmed()
            ),
        )?;
        return Ok(());
    }

    // Create the environment container. This is our "lock file" for the environment
    // within Docker -- it's the first resource we create and the last one we delete.
    let spinner = cliclack::spinner();
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
    let spinner = cliclack::spinner();
    spinner.start("Creating environment network...");
    ctx.docker()
        .api()
        .networks()
        .create(&NetworkCreateOpts::for_stackify_environment(&env_name))
        .await?;
    spinner.stop("Environment network created");

    Ok(())
}
