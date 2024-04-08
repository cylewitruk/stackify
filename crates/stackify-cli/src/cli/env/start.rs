use cliclack::{intro, log::*, multi_progress, outro_note, MultiProgress};
use color_eyre::{
    eyre::{bail, eyre},
    Result,
};
use console::style;
use docker_api::{
    models::ContainerSummary,
    opts::{
        ContainerConnectionOpts, ContainerConnectionOptsBuilder, ContainerCreateOpts,
        ContainerListOpts, NetworkCreateOpts, NetworkListOpts,
    },
};
use stackify_common::{
    docker::ContainerState,
    types::{EnvironmentName, EnvironmentService},
};

use crate::{
    cli::{context::CliContext, theme::ThemedObject},
    db::{cli_db::CliDatabase, errors::LoadEnvironmentError},
    docker::{
        format_container_name,
        opts::{CreateContainer, CreateNetwork, ListContainers, ListNetworks},
    },
    util::names::environment_container_name,
};

use super::args::StartArgs;

pub async fn exec(ctx: &CliContext, args: StartArgs) -> Result<()> {
    let env_name = EnvironmentName::new(&args.env_name)?;
    intro("Start Environment".bold())?;

    let env = match ctx.db.load_environment(&env_name) {
        Ok(env) => env,
        Err(err) => match err {
            LoadEnvironmentError::NotFound => {
                warning(format!(
                    "The {} environment does not exist.\n",
                    env_name.magenta()
                ))?;
                outro_note(
                    "Environment Not Found".bold().red(),
                    format!(
                        "{} {} {}",
                        "To create an environment, use the",
                        "stackify env create".bold().white(),
                        style("command.").dimmed()
                    ),
                )?;
                return Ok(());
            }
            LoadEnvironmentError::MissingParam {
                service_name,
                param_name,
            } => {
                warning(format!(
                    "The {} service is missing the {} parameter.\n",
                    service_name.magenta(),
                    param_name.cyan()
                ))?;
                outro_note(
                    "Configuration Error".bold().red(),
                    format!(
                        "{} {} {}",
                        "To add a parameter to the service, use the",
                        "stackify env service config".bold().white(),
                        style("command.").dimmed()
                    ),
                )?;
                return Ok(());
            }
            _ => bail!(err),
        },
    };
    // .map_err(|e| {
    //     match e.downcast_ref() {
    //         Some(LoadEnvironmentError::NotFound) => {
    //             warning(format!(
    //                 "The '{}' environment does not exist.\n",
    //                 env_name
    //             ))?;
    //             outro_note(
    //                 "Environment Not Found",
    //                 format!(
    //                     "{} {} {}",
    //                     "To create an environment, use the".bold().red(),
    //                     "stackify env create".bold().white(),
    //                     style("command.").dimmed()
    //                 ),
    //             )?;
    //             return Ok(());
    //         },
    //         _ => Err(e)
    //     }
    // })?;

    // Check if the environment has any services defined. If not, return an error.
    if env.services.is_empty() {
        warning(format!(
            "The '{}' environment has no services defined, so there is nothing to start.\n",
            env_name
        ))?;

        outro_note(
            "No Services Defined",
            format!(
                "{} {} {}",
                "To add a service to the environment, use the".bold().red(),
                "stackify env service".bold().white(),
                style("command.").dimmed()
            ),
        )?;
        return Ok(());
    }

    let multi = multi_progress("Checking environment status");

    // Assert that the environment is not already running.
    let existing_containers = ctx
        .docker()
        .api()
        .containers()
        .list(&ContainerListOpts::for_environment(&env_name, true))
        .await?;

    assert_network(ctx, &multi, &env_name).await?;
    assert_environment_container(ctx, &multi, &env_name, &existing_containers).await?;

    multi.stop();

    outro_note(
        "Environment Started".bold().green(),
        format!(
            "{} {} {}",
            style("To see the environment status, use the"),
            style("stackify env status").bold().white(),
            style("command.").dimmed()
        ),
    )?;

    Ok(())
}

async fn assert_network(
    ctx: &CliContext,
    multi: &MultiProgress,
    env_name: &EnvironmentName,
) -> Result<()> {
    // Create the network for the environment.
    let spinner = multi.add(cliclack::spinner());
    spinner.start("Docker network...");
    let existing_network = ctx
        .docker()
        .api()
        .networks()
        .list(&NetworkListOpts::for_environment(&env_name))
        .await?;

    if existing_network.len() == 0 {
        ctx.docker()
            .api()
            .networks()
            .create(&NetworkCreateOpts::for_stackify_environment(&env_name))
            .await?;
        spinner.stop(format!("{} Docker network", "✔".green()));
    } else if existing_network.len() == 1 {
        spinner.stop(format!("{} Docker network", "✔".green()));
    } else {
        spinner.error("Multiple networks found for the environment");
        outro_note(
            "Multiple Networks".red().bold(),
            format!(
                "There are multiple networks for the environment. Please remove the extra networks and try again."
            ),
        )?;
        return Ok(());
    }

    Ok(())
}

async fn assert_environment_container(
    ctx: &CliContext,
    multi: &MultiProgress,
    env_name: &EnvironmentName,
    existing_containers: &[ContainerSummary],
) -> Result<()> {
    let spinner = multi.add(cliclack::spinner());
    spinner.start("Environment container...");
    let env_container = existing_containers.iter().find(|c| {
        c.names
            .as_ref()
            .unwrap()
            .contains(&environment_container_name(&env_name))
    });
    if let Some(container) = env_container {
        if let Some(state) = &container.state {
            if ContainerState::parse(state)? == ContainerState::Running {
                spinner.error(format!(
                    "The environment {} is already running.",
                    env_name.magenta().bold()
                ));
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
            } else if ContainerState::parse(state)? == ContainerState::Created {
                spinner.stop(format!("{} Environment container", "✔".green()));
            }
        }
    } else {
        // Create the environment container. This is our "lock file" for the environment
        // within Docker -- it's the first resource we create and the last one we delete.
        ctx.docker()
            .api()
            .containers()
            .create(&ContainerCreateOpts::for_stackify_environment_container(
                &env_name,
            ))
            .await?;
        spinner.stop(format!("{} Environment container", "✔".green()));
    }

    Ok(())
}

async fn start_bitcoin_miner(
    ctx: &CliContext,
    multi: &MultiProgress,
    env_name: &EnvironmentName,
    service: &EnvironmentService,
) -> Result<()> {
    // Format the container name
    let container_name = format_container_name(env_name, &service.name);

    // Begin the progress spinner
    let spinner = multi.add(cliclack::spinner());
    spinner.start(format!("Starting {}...", container_name));

    if let Some((id, _)) = ctx.docker().find_container_by_name(&container_name).await? {
        // If the container already exists, start it.
        ctx.docker().api().containers().get(id).start().await?;
    } else {
        // Otherwise, create the container
        let container = ctx
            .docker()
            .api()
            .containers()
            .create(
                &ctx.docker()
                    .opts_for()
                    .create_bitcoin_miner_container(&env_name, &service),
            )
            .await?;

        container
            .copy_file_into(
                ctx.docker()
                    .container_dirs()
                    .home_dir
                    .join(".bitcoin/bitcoin.conf"),
                &[],
            )
            .await?;

        // Attach the container to this environment's network
        if let Some((network_id, _)) = ctx.docker().find_network_for_environment(env_name).await? {
            ctx.docker()
                .api()
                .networks()
                .get(network_id)
                .connect(&ContainerConnectionOpts::builder(container.id()).build())
                .await?;
        }

        // Start the container
        container.start().await?;
    }

    spinner.stop(format!("{} {}", "✔".green(), container_name));

    Ok(())
}
