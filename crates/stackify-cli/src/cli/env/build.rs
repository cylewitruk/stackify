use cliclack::{intro, multi_progress, outro, outro_cancel, spinner};
use color_eyre::eyre::eyre;
use color_eyre::Result;
use console::style;
use docker_api::conn::TtyChunk;
use docker_api::models::ContainerState;
use docker_api::opts::{ContainerCreateOpts, ContainerStopOpts, LogsOpts};
use docker_api::Container;
use futures_util::StreamExt;
use stackify_common::types::{EnvironmentName, EnvironmentService};
use stackify_common::ServiceType;
use std::collections::HashMap;

use crate::cli::context::CliContext;
use crate::cli::theme::ThemedObject;
use crate::db::cli_db::CliDatabase as _;
use crate::docker::opts::CreateContainer;

use super::args::BuildArgs;

pub async fn exec(ctx: &CliContext, args: BuildArgs) -> Result<()> {
    let db = ctx.db.as_clidb();
    let env_name = EnvironmentName::new(&args.env_name)?;
    let env = db
        .load_environment(&env_name)?
        .ok_or(eyre!("The '{}' environment does not exist.", env_name))?;

    intro(format!(
        "{} {}",
        "Building Environment".bold(),
        env_name.bold().magenta()
    ))?;

    if env.services.is_empty() {
        cliclack::log::warning(format!(
            "The '{}' environment has no services defined, so there is nothing to build.
            Please define at least one service before attempting to build the environment.",
            env_name
        ))?;

        outro_cancel(format!(
            "See the {} command for more information.",
            style("stackify env service").white().bold()
        ))?;

        return Ok(());
    }

    ctx.register_shutdown(|d| async move {
        if let Some((container_id, _)) = d.find_container_by_name("stackify-build").await.unwrap() {
            let container = d.api().containers().get(container_id);
            let inspect = container.inspect().await.unwrap();

            if let Some(ContainerState {
                running: Some(true),
                ..
            }) = inspect.state
            {
                container
                    .stop(&ContainerStopOpts::builder().signal("SIGKILL").build())
                    .await
                    .unwrap();
            }
            container.delete().await.unwrap();
        }
    })
    .await;

    // Iterate through each service in the environment and build them.
    for service in env.services.iter() {
        //let service = &env.services[1];
        let multi = multi_progress(format!("Building service {}", service.name.magenta()));

        // Check for an existing build container and stop/remove it if found
        kill_existing_build_container(&ctx, &multi).await?;

        // Create the build container
        let build = create_build_container(&ctx, &multi, &service).await?;

        // Start the build
        start_build(&multi, &build).await?;

        // Follow logs and show progress
        follow_build_logs(&multi, &build).await?;

        // Stop & remove the build container, ignoring errors
        let _ = build.stop(&ContainerStopOpts::default()).await;
        let _ = build.delete().await;

        multi.stop();
    }

    outro("Finished building the environment")?;

    Ok(())
}

async fn kill_existing_build_container(
    ctx: &CliContext,
    multi: &cliclack::MultiProgress,
) -> Result<()> {
    let kill_container_spinner = multi.add(spinner());
    kill_container_spinner.start("Checking for existing Stackify build container...");

    if let Some((container_id, _)) = ctx
        .docker()
        .find_container_by_name("stackify-build")
        .await?
    {
        kill_container_spinner.start(format!(
            "Removing existing Stackify build container {}...",
            container_id
        ));
        let container = ctx.docker().api().containers().get(container_id);
        let inspect = container.inspect().await?;

        if let Some(ContainerState {
            running: Some(true),
            ..
        }) = inspect.state
        {
            container.stop(&ContainerStopOpts::default()).await?;
        }
        container.delete().await?;

        kill_container_spinner.stop("Existing Stackify build container removed");
    } else {
        kill_container_spinner.stop("No existing Stackify build container found");
        kill_container_spinner.clear();
    }

    Ok(())
}

async fn create_build_container(
    ctx: &CliContext,
    multi: &cliclack::MultiProgress,
    service: &EnvironmentService,
) -> Result<Container> {
    let create_container_spinner = multi.add(spinner());
    create_container_spinner.start("Creating build container...");

    let mut env_vars = HashMap::<String, String>::new();
    if ServiceType::StacksMiner.is(service.service_type.id) {
        let target = service
            .version
            .git_target
            .as_ref()
            .map(|x| x.target.clone())
            .unwrap_or_default();
        eprintln!("BUILD STACKS! {}", target);
        env_vars.insert("BUILD_STACKS".into(), target);
    }

    let create_opts = ContainerCreateOpts::for_stackify_build_container(
        &ctx.host_dirs.bin_dir,
        &ctx.host_dirs.assets_dir,
        env_vars,
    );

    let build = ctx.docker().api().containers().create(&create_opts).await?;
    create_container_spinner.stop("Build container created");

    Ok(build)
}

async fn start_build(multi: &cliclack::MultiProgress, build: &Container) -> Result<()> {
    let start_container_spinner = multi.add(spinner());
    start_container_spinner.start("Starting build container...");
    build.start().await?;
    start_container_spinner.stop("Build container started");

    Ok(())
}

async fn follow_build_logs(multi: &cliclack::MultiProgress, build: &Container) -> Result<()> {
    let build_service_spinner = multi.add(spinner());
    build_service_spinner.start("Building service...");

    let mut log_stream = build.logs(
        &LogsOpts::builder()
            .stderr(true)
            .stdout(true)
            .follow(true)
            .all()
            .build(),
    );

    let building_text = style("Building").bold().cyan();

    while let Some(log) = log_stream.next().await {
        match log {
            Ok(TtyChunk::StdOut(chunk)) => {
                let msg = String::from_utf8_lossy(&chunk);
                let msg = msg.strip_suffix("\n").unwrap_or(&msg);
                build_service_spinner.set_message(format!("{building_text}  {msg}"));
            }
            Ok(TtyChunk::StdErr(chunk)) => {
                let msg = String::from_utf8_lossy(&chunk);
                let msg = msg.strip_suffix("\n").unwrap_or(&msg);
                build_service_spinner.set_message(format!("{building_text} {msg}"));
            }
            Ok(TtyChunk::StdIn(_)) => unreachable!(),
            Err(_) => {
                //cliclack::log::error(format!("Error reading log stream: {}", e))?;
            }
        }
    }

    build_service_spinner.stop("Service built");

    Ok(())
}
