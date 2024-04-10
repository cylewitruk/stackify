use crate::{cli::log::clilog, docker::ContainerState, util::names::service_container_name};
use cliclack::{intro, log::*, multi_progress, outro_note, MultiProgress};
use color_eyre::{eyre::bail, Result};
use console::style;
use docker_api::{
    opts::{
        ContainerConnectionOpts, ContainerCreateOpts, ContainerListOpts, NetworkCreateOpts,
        NetworkListOpts,
    },
    Container,
};
use handlebars::Handlebars;
use stackify_common::{
    types::{EnvironmentName, EnvironmentService},
    FileType, ServiceType,
};

use crate::{
    cli::{context::CliContext, theme::ThemedObject},
    db::cli_db::CliDatabase,
    docker::opts::{CreateContainer, CreateNetwork, ListContainers, ListNetworks},
    util::names::environment_container_name,
};

use super::args::StartArgs;

pub async fn exec(ctx: &CliContext, args: StartArgs) -> Result<()> {
    intro("Start Environment".bold())?;
    let env_name = EnvironmentName::new(&args.env_name)?;

    let env = ctx.db.load_environment(&env_name)?;

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

    clilog!("Existing containers: {:?}", existing_containers);

    assert_network(ctx, &multi, &env_name).await?;
    let env_container = assert_environment_container(ctx, &multi, &env_name).await?;

    multi.stop();
    let multi = multi_progress("Starting environment services");

    // Start the environment services
    // Starting with the `environment` container...
    let spinner = multi.add(cliclack::spinner());
    spinner.start(environment_container_name(&env_name));
    env_container.start().await?;
    spinner.stop(format!(
        "{} {}",
        "✔".green(),
        environment_container_name(&env_name)
    ));

    let bitcoin_service_names = env
        .services
        .iter()
        .filter(|service| {
            [ServiceType::BitcoinMiner, ServiceType::BitcoinFollower]
                .contains(&ServiceType::from_i32(service.service_type.id).unwrap())
        })
        .map(|service| service.name.clone())
        .collect::<Vec<_>>();

    for service in env.services {
        match ServiceType::from_i32(service.service_type.id)? {
            ServiceType::BitcoinMiner | ServiceType::BitcoinFollower => {
                let bitcoin_peers = bitcoin_service_names
                    .iter()
                    .filter(|name| *name != &service.name)
                    .cloned()
                    .collect::<Vec<_>>();

                start_bitcoin_node(ctx, &multi, &env_name, &service, &bitcoin_peers).await?;
            }
            _ => clilog!(
                "Service type {} is not yet supported, skipping...",
                service.service_type.name
            ),
        }
    }

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
) -> Result<Container> {
    let spinner = multi.add(cliclack::spinner());
    spinner.start("Environment container...");

    let env_container = ctx
        .docker()
        .find_container_by_name(&environment_container_name(&env_name))
        .await?;

    if let Some((id, container)) = env_container {
        let container_ref = ctx.docker().api().containers().get(id);
        // If the container already exists, check if it's running.
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
            } else if ContainerState::parse(state)? == ContainerState::Created {
                spinner.stop(format!("{} Environment container", "✔".green()));
            }
            return Ok(container_ref);
        } else {
            panic!("Environment container is in an unknown state.");
        }
    } else {
        // Create the environment container. This is our "lock file" for the environment
        // within Docker -- it's the first resource we create and the last one we delete.
        let container = ctx
            .docker()
            .api()
            .containers()
            .create(&ContainerCreateOpts::for_stackify_environment_container(
                &env_name,
            ))
            .await?;
        spinner.stop(format!("{} Environment container", "✔".green()));
        Ok(container)
    }
}

async fn start_bitcoin_node(
    ctx: &CliContext,
    multi: &MultiProgress,
    env_name: &EnvironmentName,
    service: &EnvironmentService,
    bitcoin_peers: &[String],
) -> Result<()> {
    // Format the container name
    let container_name = service_container_name(&service);
    clilog!("Starting: {}", &container_name);

    // Begin the progress spinner
    let spinner = multi.add(cliclack::spinner());
    spinner.start(format!("Starting {}...", &container_name));

    if let Some((id, _)) = ctx.docker().find_container_by_name(&container_name).await? {
        clilog!(
            "Container '{}' already exists; starting...",
            &container_name
        );
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
                    .create_bitcoin_container(&env_name, &service)?,
            )
            .await?;

        clilog!("Handling files for service: {}", &service.name);
        let handlebars = Handlebars::new();
        let mut data = serde_json::Map::new();

        for file in ctx.db.load_files_for_environment_service(service)? {
            clilog!("Handling file: {}", &file.header.filename);
            data.insert("peers".to_string(), bitcoin_peers.into());
            let mut content = file.contents.contents;

            if file.header.file_type == FileType::HandlebarsTemplate {
                let rendered_content =
                    handlebars.render_template(&String::from_utf8(content)?, &data)?;
                content = rendered_content.into_bytes();
            }

            let destination_path = &file.header.destination_dir.join(&file.header.filename);
            clilog!(
                "Copying file: {} -> {:?}",
                &file.header.filename,
                destination_path
            );
            container.copy_file_into(destination_path, &content).await?;
        }

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

    spinner.stop(format!("{} {}", "✔".green(), &container_name));

    Ok(())
}
