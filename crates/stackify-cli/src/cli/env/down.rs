use cliclack::{intro, multi_progress, outro};
use color_eyre::{eyre::eyre, Result};
use stackify_common::types::EnvironmentName;

use crate::{
    docker_api::opts::{ContainerListOpts, ContainerStopOpts},
    cli::{context::CliContext, theme::ThemedObject},
    docker::{network_name, opts::ListContainers, ContainerState},
};

use super::args::DownArgs;

pub async fn exec(ctx: &CliContext, args: DownArgs) -> Result<()> {
    intro("Tear-Down Environment".bold())?;
    let env_name = EnvironmentName::new(&args.env_name)?;

    remove_containers(ctx, &env_name).await?;
    remove_network(ctx, &env_name).await?;

    outro("Finished!".bold().green())?;

    Ok(())
}

async fn remove_containers(ctx: &CliContext, env_name: &EnvironmentName) -> Result<()> {
    let multi = multi_progress("Removing containers");

    let containers = ctx
        .docker()
        .api()
        .containers()
        .list(&ContainerListOpts::for_environment(env_name, true))
        .await?;

    if containers.is_empty() {
        let spinner = multi.add(cliclack::spinner());
        spinner.stop(format!(
            "{} {}",
            "✔".green(),
            "This environment has no containers"
        ));
        multi.stop();
        return Ok(());
    }

    for summary in containers {
        let spinner = multi.add(cliclack::spinner());
        let container_id = summary.id.ok_or(eyre!("Container ID not found."))?;
        let container_names = summary.names.ok_or(eyre!("Container name not found."))?;
        let container_name = container_names.join(", ");
        spinner.start(format!("Removing container: {}", container_name));

        let container = ctx.docker().api().containers().get(container_id);

        if let Some(state) = summary.state.as_deref() {
            spinner.set_message(format!("Stopping container: {}", container_name));
            if ContainerState::parse(state)? == ContainerState::Running {
                container
                    .stop(&ContainerStopOpts::builder().signal("SIGKILL").build())
                    .await?;
            }
        }

        container.delete().await?;

        spinner.stop(format!(
            "{} Container {} removed",
            "✔".green(),
            container_name.magenta()
        ));
    }

    multi.stop();

    Ok(())
}

async fn remove_network(ctx: &CliContext, env_name: &EnvironmentName) -> Result<()> {
    let multi = multi_progress("Removing network");

    let network_name = network_name(env_name);
    let spinner = multi.add(cliclack::spinner());
    spinner.start(format!("Removing network: {}", &network_name));

    if let Some((id, _)) = ctx.docker().find_network_for_environment(env_name).await? {
        ctx.docker().api().networks().get(id).delete().await?;

        spinner.stop(format!(
            "{} Network {} removed",
            "✔".green(),
            &network_name.magenta()
        ));
    } else {
        spinner.stop(format!(
            "{} Network {} does not exist",
            "✔".green(),
            &network_name.magenta()
        ));
    }

    multi.stop();

    Ok(())
}
