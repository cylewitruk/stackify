use cliclack::{intro, log::*, outro, outro_note};
use color_eyre::{eyre::eyre, Result};
use console::style;
use docker_api::opts::{ContainerListOpts, ContainerStopOpts};
use stackify_common::types::EnvironmentName;

use crate::{
    cli::{context::CliContext, theme::ThemedObject},
    docker::opts::ListContainers,
};

use super::args::StopArgs;

pub async fn exec(ctx: &CliContext, args: StopArgs) -> Result<()> {
    let env_name = EnvironmentName::new(&args.env_name)?;

    intro("Stop Environment".bold())?;

    let containers = ctx
        .docker()
        .api()
        .containers()
        .list(&ContainerListOpts::running_in_environment(&env_name))
        .await?;

    if containers.is_empty() {
        info("There are no running containers for the environment.\n")?;
        outro_note(
            "No Running Containers",
            format!(
                "To start the environment, use the {} command.",
                style("stackify env start").white().bold()
            ),
        )?;
        return Ok(());
    }

    for container in containers {
        let spinner = cliclack::spinner();
        let container_id = container.id.ok_or(eyre!("Container ID not found."))?;
        let container_names = container.names.ok_or(eyre!("Container name not found."))?;
        let container_name = container_names.join(", ");
        spinner.start(format!("Stopping container {}", container_name.cyan()));
        ctx.docker()
            .api()
            .containers()
            .get(container_id)
            .stop(&ContainerStopOpts::default())
            .await?;
        spinner.stop(format!(
            "{} Container {} stopped",
            style("✔").green(),
            container_name
        ));
    }

    Ok(())
}
