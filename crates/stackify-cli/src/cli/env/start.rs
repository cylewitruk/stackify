use crate::{
    cli::{log::clilog, Cli},
    docker::{api::DockerApi, ContainerState},
    util::{names::service_container_name, stacks_cli::MakeKeychainResult},
};
use cliclack::{intro, log::*, multi_progress, outro_note, MultiProgress};
use color_eyre::{
    eyre::{bail, eyre},
    Result,
};
use console::style;
use docker_api::{
    opts::{
        ContainerConnectionOpts, ContainerCreateOpts, ContainerListOpts, NetworkCreateOpts,
        NetworkListOpts,
    },
    Container,
};
use handlebars::{to_json, Handlebars};
use stackify_common::{
    types::{Environment, EnvironmentName, EnvironmentService},
    FileType, ServiceType, ValueType,
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
            "No Services Defined".red().bold(),
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

    if existing_containers.len() > 0 {
        multi.stop();
        outro_note(
            "Environment Running".red().bold(),
            format!("Some containers for the environment are already running. Please stop the environment before starting it again."),
        )?;
        return Ok(());
    }

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

    for service in &env.services {
        match ServiceType::from_i32(service.service_type.id)? {
            ServiceType::BitcoinMiner | ServiceType::BitcoinFollower => {
                start_bitcoin_node(ctx, &multi, &env, service).await?;
            }
            ServiceType::StacksMiner | ServiceType::StacksFollower => {
                start_stacks_node(ctx, &multi, &env, service).await?;
            }
            ServiceType::StacksSigner => {
                start_stacks_signer(ctx, &multi, &env, service).await?;
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

/// Assert that the Docker network for the environment exists, and if not create
/// it.
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

/// Assert that the environment container exists, and if not create it.
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

/// Start a Stacks Signer node. If a container has already been created for this
/// service then it will be re-used and started from its shut-down state, 
/// otherwise a new container will be created.
async fn start_stacks_signer(
    ctx: &CliContext,
    multi: &MultiProgress,
    env: &Environment,
    service: &EnvironmentService,
) -> Result<()> {
    let container_name = service_container_name(&service);
    clilog!("Starting: {}", &container_name);

    let spinner = multi.add(cliclack::spinner());
    spinner.start(format!("Starting {}...", &container_name));

    // Check for an existing container, and create one if it doesn't exist.
    let container = match try_get_container(ctx.docker(), &container_name).await? {
        Some(container) => container,
        None => create_stacks_signer_container(ctx, env, service).await?,
    };

    // Attach the container to the environment's network.
    if let Some((network_id, _)) = ctx.docker().find_network_for_environment(&env.name).await? {
        ctx.docker()
            .api()
            .networks()
            .get(network_id)
            .connect(&ContainerConnectionOpts::builder(container.id()).build())
            .await?;
    }

    container.start().await?;
    spinner.stop(format!("{} {}", "✔".green(), &container_name));

    Ok(())
}

/// Start a Stacks Node. If a container has already been created for this
/// service then it will be re-used and started from its shut-down state,
/// otherwise a new container will be created.
async fn start_stacks_node(
    ctx: &CliContext,
    multi: &MultiProgress,
    env: &Environment,
    service: &EnvironmentService,
) -> Result<()> {
    let container_name = service_container_name(&service);
    clilog!("Starting: {}", &container_name);

    let spinner = multi.add(cliclack::spinner());
    spinner.start(format!("Starting {}...", &container_name));

    let container = match try_get_container(ctx.docker(), &container_name).await? {
        Some(container) => container,
        None => create_stacks_node_container(ctx, ctx.docker(), env, service).await?,
    };

    if let Some((network_id, _)) = ctx.docker().find_network_for_environment(&env.name).await? {
        ctx.docker()
            .api()
            .networks()
            .get(network_id)
            .connect(&ContainerConnectionOpts::builder(container.id()).build())
            .await?;
    }

    container.start().await?;
    spinner.stop(format!("{} {}", "✔".green(), &container_name));

    Ok(())
}

/// Attempt to get a container by name. If the container is found, it will be
/// returned as a `Some(Container)`, otherwise `None` will be returned.
async fn try_get_container(docker: &DockerApi, container_name: &str) -> Result<Option<Container>> {
    if let Some((id, _)) = docker.find_container_by_name(&container_name).await? {
        Ok(Some(docker.api().containers().get(id)))
    } else {
        Ok(None)
    }
}

/// Start a Bitcoin node. If a container has already been created for this
/// service then it will be re-used and started from its shut-down state,
/// otherwise a new container will be created.
async fn start_bitcoin_node(
    ctx: &CliContext,
    multi: &MultiProgress,
    env: &Environment,
    service: &EnvironmentService,
) -> Result<()> {
    // Format the container name
    let container_name = service_container_name(&service);
    clilog!("Starting: {}", &container_name);

    // Begin the progress spinner
    let spinner = multi.add(cliclack::spinner());
    spinner.start(format!("Starting {}...", &container_name));

    let container = match try_get_container(ctx.docker(), &container_name).await? {
        Some(container) => container,
        None => {
            let container = create_bitcoin_container(ctx.docker(), &env.name, service).await?;

            let bitcoin_peers = env
                .services
                .iter()
                .filter(|service| {
                    [ServiceType::BitcoinMiner, ServiceType::BitcoinFollower]
                        .contains(&ServiceType::from_i32(service.service_type.id).unwrap())
                })
                .filter(|svc| &svc.name != &service.name)
                .map(|service| service.name.clone())
                .collect::<Vec<_>>();

            clilog!("Handling files for service: {}", &service.name);
            let handlebars = Handlebars::new();
            let mut data = serde_json::Map::new();

            for file in ctx.db.load_files_for_environment_service(service)? {
                clilog!("Handling file: {}", &file.header.filename);
                data.insert("peers".to_string(), to_json(&bitcoin_peers));
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
            if let Some((network_id, _)) =
                ctx.docker().find_network_for_environment(&env.name).await?
            {
                ctx.docker()
                    .api()
                    .networks()
                    .get(network_id)
                    .connect(&ContainerConnectionOpts::builder(container.id()).build())
                    .await?;
            }

            container
        }
    };

    // Start the container
    container.start().await?;

    spinner.stop(format!("{} {}", "✔".green(), &container_name));

    Ok(())
}

/// Create a new Bitcoin container for the environment.
async fn create_bitcoin_container(
    docker: &DockerApi,
    env_name: &EnvironmentName,
    service: &EnvironmentService,
) -> Result<Container> {
    Ok(docker
        .api()
        .containers()
        .create(
            &docker
                .opts_for()
                .create_bitcoin_container(&env_name, &service)?,
        )
        .await?)
}

/// Create a new Stacks Signer container for the environment.
async fn create_stacks_signer_container(
    ctx: &CliContext,
    env: &Environment,
    service: &EnvironmentService,
) -> Result<Container> {
    let docker = ctx.docker();

    let container = docker
        .api()
        .containers()
        .create(
            &docker
                .opts_for()
                .create_stacks_signer_container(&env.name, &service)?,
        )
        .await?;

    clilog!("Handling files for service: {}", &service.name);
    let handlebars = Handlebars::new();
    let mut data = serde_json::Map::new();

    let stacks_node_endpoint = service.params.iter()
        .find(|param| param.param.key == "stacks_node")
        .expect("Stacks node endpoint not found for Stacks signer");

    let keychain = service.params.iter()
        .find(|param| param.param.key == "stacks_keychain")
        .map(|param| MakeKeychainResult::from_json(&param.value))
        .expect("Keychain not found for Stacks signer")?;

    data.insert("stacks_node_endpoint".to_string(), to_json(&format!("{}:20443", stacks_node_endpoint.value)));
    data.insert("signer_private_key".to_string(), to_json(&keychain.key_info.private_key));

    clilog!("Generating configuration files for service: {}", &service.name);
    for file in ctx.db.load_files_for_environment_service(service)? {
        clilog!("Processing file: {}", &file.header.filename);
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

    clilog!("Container data: {:?}", data);

    Ok(container)
}

/// Create a new Stacks node container for the environment.
async fn create_stacks_node_container(
    ctx: &CliContext,
    docker: &DockerApi,
    env: &Environment,
    service: &EnvironmentService,
) -> Result<Container> {
    let container = docker
        .api()
        .containers()
        .create(
            &docker
                .opts_for()
                .create_stacks_node_container(&env.name, &service)?,
        )
        .await?;

    clilog!("Handling files for service: {}", &service.name);
    let handlebars = Handlebars::new();
    let mut data = serde_json::Map::new();

    // Find other miners in the environment and use them as bootstrap nodes.
    // TODO: Add a param to miner configuration to set if it is the environment bootstrap node.
    let stacks_bootstrap_nodes = env.find_service_instances(ServiceType::StacksMiner, &service.name)
        .iter()
        .map(|service| {
            let seed = service
                .params
                .iter()
                .find(|param| param.param.key == "stacks_keychain")
                .expect("Seed param not found for Stacks miner");
            let keychain = MakeKeychainResult::from_json(&seed.value)
                .expect("Keychain not found for Stacks miner");
            let miner_pubkey = keychain.key_info.public_key;
            format!("{miner_pubkey}@{}:20444", service.name.clone())
        })
        .collect::<Vec<_>>();

    data.insert(
        "bootstrap_node".to_string(),
        to_json(stacks_bootstrap_nodes.first()),
    );

    // Find Bitcoin nodes in the environment.
    clilog!("Finding Bitcoin peers in the environment...");
    let bitcoin_nodes = env
        .services
        .iter()
        .filter(|service| {
            [ServiceType::BitcoinMiner, ServiceType::BitcoinFollower]
                .contains(&ServiceType::from_i32(service.service_type.id).unwrap())
        })
        .filter(|svc| &svc.name != &service.name)
        .map(|service| service.name.clone())
        .collect::<Vec<_>>();

    // It is an invalid configuration to not have specified a Bitcoin peer. The
    // peer doesn't have to be reachable, but it must be provided.
    if bitcoin_nodes.is_empty() {
        bail!("Invalid environment configuration: no Bitcoin peers found");
    }

    // Add the first Bitcoin node as the node's burnchain peer.
    clilog!("Bitcoin neighbor selected: {:?}", bitcoin_nodes.first());
    data.insert(
        "burnchain_peer_host".to_string(),
        to_json(bitcoin_nodes.first()),
    );

    clilog!("Checking for Stacks signers configured to use this node as a peer...");
    let signer_nodes = env
        .services
        .iter()
        .filter(|svc| {
            // Stacks signer nodes only
            [ServiceType::StacksSigner]
                .contains(&ServiceType::from_i32(svc.service_type.id).unwrap())
            // ...and which have their 'stacks_node' parameter set to this node
            && svc.params
                .iter()
                .any(|param| param.param.key == "stacks_node" && &param.value == &service.name)
        })
        .map(|svc| svc.name.clone())
        .collect::<Vec<_>>();
    if signer_nodes.is_empty() {
        clilog!("No Stacks signers configured to use this node as a peer.");
    }

    // Add event observers.
    let mut event_observers = Vec::new();
    // Add any signers configured to use this node as a peer to the event observers.
    for signer in signer_nodes {
        // [[events_observer]]
        // endpoint = "{{e.endpoint}}"
        // retry_count = {{e.retry_count}}
        // include_data_events = {{e.include_data_events}}
        // events_keys = [{{#each e.events_keys as |k|}}"{{k}}",{{/each}}]
        clilog!("Adding signer node: {}", &signer);
        let mut observer = serde_json::Map::new();
        observer.insert("endpoint".to_string(), to_json(format!("{}:30000", &signer)));
        observer.insert("retry_count".to_string(), to_json(255));
        observer.insert("include_data_events".to_string(), to_json(false));
        observer.insert("event_keys".to_string(), to_json(vec!["stackerdb", "block_proposal", "burn_blocks"]));
        event_observers.push(to_json(observer));
    }
    data.insert("event_observers".to_string(), to_json(event_observers));

    // If the node is configured as a miner, include mining configuration.
    if ServiceType::from_i32(service.service_type.id)? == ServiceType::StacksMiner {
        data.insert("miner".to_string(), to_json(true));
    } else {
        data.insert("miner".to_string(), to_json(false));
    }

    // Add configuration parameters.
    clilog!("Processing configuration parameters for service: {}", &service.name);
    for param in &service.params {
        clilog!("Inserting param: {:?}", param);
        match param.param.value_type {
            ValueType::String => {
                data.insert(param.param.key.clone(), to_json(&param.value));
            }
            ValueType::Integer => {
                data.insert(
                    param.param.key.clone(),
                    to_json(param.value.parse::<i64>()?),
                );
            }
            ValueType::Boolean => {
                data.insert(
                    param.param.key.clone(),
                    to_json(param.value.parse::<bool>()?),
                );
            }
            ValueType::StacksKeychain => {
                clilog!(
                    "Inserting keychain param: {}: {}",
                    &param.param.key,
                    &param.value
                );
                data.insert(
                    param.param.key.clone(),
                    to_json(MakeKeychainResult::from_json(&param.value)?),
                );
            }
            _ => bail!("Unsupported value type: {:?}", param.param.value_type),
        }
    }

    // Replace values in configuration files.
    clilog!("Generating configuration files for service: {}", &service.name);
    for file in ctx.db.load_files_for_environment_service(service)? {
        clilog!("Processing file: {}", &file.header.filename);
        let mut content = file.contents.contents;

        if file.header.file_type == FileType::HandlebarsTemplate {
            let rendered_content =
                handlebars.render_template(&String::from_utf8(content)?, &data)?;
            clilog!("Rendered content: {}", &rendered_content);
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

    clilog!("Container data: {:?}", data);

    Ok(container)
}
