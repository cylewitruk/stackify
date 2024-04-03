use cliclack::{intro, outro};
use color_eyre::eyre::eyre;
use color_eyre::Result;
use console::style;
use docker_api::opts::ContainerCreateOpts;
use futures_util::StreamExt;
use stackify_common::types::EnvironmentName;

use crate::cli::theme::ThemedObject;
use crate::cli::{context::CliContext, warn};
use crate::db::cli_db::CliDatabase as _;
use crate::docker::opts::CreateContainer;
use crate::includes::STACKIFY_BUILD_ENTRYPOINT;

use super::args::BuildArgs;

pub async fn exec(ctx: &CliContext, args: BuildArgs) -> Result<()> {
    let db = ctx.db.as_clidb();
    let env_name = EnvironmentName::new(&args.env_name)?;
    let env = db
        .load_environment(&env_name)?
        .ok_or(eyre!("The '{}' environment does not exist.", env_name))?;

    intro(format!(
        "Building the environment {}",
        env_name.bold().magenta()
    ))?;

    if env.services.is_empty() {
        warn(format!(
            "The '{}' environment has no services defined, so there is nothing to start.",
            env_name
        ));
        println!("Please define at least one service before starting the environment.");
        println!(
            "See the {} command for more information.",
            style("stackify env service").white().bold()
        );
        return Ok(());
    }

    if let Some((container_id, container)) = ctx
        .docker()
        .find_container_by_name("/stackify-build")
        .await?
    {
        cliclack::log::info(format!("Removing existing Stackify build container."))?;
        //ctx.docker.rm_container(&container.id)?;
        ctx.docker().delete_container("/stackify-build").await?;
        cliclack::log::info(format!("Stackify build container removed."))?;
    }

    for service in env.services {
        let mut spinner = cliclack::spinner();
        spinner.start("Creating new Stackify build container...");
        let create_opts =
            ContainerCreateOpts::for_stackify_build_container(&ctx.bin_dir, &ctx.assets_dir);
        ctx.docker().api().containers().create(&create_opts).await?;
        spinner.stop("Build container created")
        // let container = ctx.docker.create_stackify_build_container(
        //     &ctx.bin_dir,
        //     &ctx.assets_dir,
        //     STACKIFY_BUILD_ENTRYPOINT,
        //     false,
        //     false,
        //     service.version.git_target.map(|x| x.target),
        //     false
        // )?;
        // cliclack::log::info(format!("Starting Stackify build container"))?;
        // ctx.docker.start_build_container()?;
        // cliclack::log::info(format!("Stackify build container started."))?;
        // let log_stream = ctx.docker.stream_container_logs(&container.id)?;
        // let runtime = tokio::runtime::Runtime::new()?;
        // runtime.block_on(async {
        //     log_stream
        //         .for_each(|log| async {
        //             cliclack::log::info(log.unwrap().message).unwrap();
        //         })
        //         .await;
        // });

        // cliclack::log::info("Stopping Stackify build container.")?;
        // ctx.docker.stop_container(&container.id)?;
        // cliclack::log::info("Stackify build container stopped.")?;
        // cliclack::log::info("Removing Stackify build container.")?;
        // ctx.docker.rm_container(&container.id)?;
        // cliclack::log::info("Stackify build container removed.")?;
    }

    outro("Finished building the environment.")?;

    Ok(())
}
