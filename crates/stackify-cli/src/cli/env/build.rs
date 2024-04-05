use cliclack::{intro, multi_progress, outro_cancel, outro_note, ProgressBar};
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
use crate::docker::ActionResult;

use super::args::BuildArgs;

pub async fn exec(ctx: &CliContext, args: BuildArgs) -> Result<()> {
    let db = ctx.db.as_clidb();
    let env_name = EnvironmentName::new(&args.env_name)?;
    let env = db
        .load_environment(&env_name)?
        .ok_or(eyre!("The '{}' environment does not exist.", env_name))?;

    intro(format!("{}", "Build Environment".bold()))?;

    cliclack::log::remark(format!(
        "Building the {env_name} environment...\n{note}\n{cancel}",
        env_name = env_name.bold().magenta(),
        note = "Note that this may take some time depending on the number and type of services."
            .dimmed(),
        cancel = "You can cancel the build at any time by pressing Ctrl+C.".dimmed()
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

    // Register our ctrl-c handler to stop and remove the build container if
    // the user cancels the build.
    register_shutdown(ctx).await;

    let mut last_result = ActionResult::Success;

    // Iterate through each service in the environment and build them.
    for service in env.services.iter() {
        if ![
            ServiceType::StacksMiner,
            ServiceType::StacksFollower,
            ServiceType::StacksSigner,
        ]
        .contains(&ServiceType::from_i32(service.service_type.id)?)
        {
            cliclack::log::step(format!(
                "Skipping {service_name} {service_type} {reason}",
                service_name = service.name.cyan(),
                service_type = format!("[{}] »", service.service_type.name).dimmed(),
                reason = "nothing to build".dimmed()
            ))?;
            continue;
        }

        let multi = multi_progress(format!(
            "Building {} {}",
            service.name.cyan(),
            format!("[{}]", service.service_type.name).dimmed()
        ));

        let spinner = multi.add(cliclack::spinner());
        spinner.start("Preparing build environment...");

        // Check for an existing build container and stop/remove it if found
        kill_existing_build_container(&ctx).await?;

        // Create the build container
        let container = create_build_container(&ctx, &service).await?;
        spinner.stop(format!("{} {}", "✔".green(), "Build environment ready"));

        // Start the build
        let spinner = multi.add(cliclack::spinner());
        spinner.start("Building service...");
        container.start().await?;

        // Follow logs and show progress
        last_result = follow_build_logs(ctx, &container, &spinner).await?;
        if !matches!(last_result, ActionResult::Success) {
            break;
        }

        // Stop & remove the build container, ignoring errors
        if !ctx.cancellation_token.is_cancelled() {
            let _ = container.stop(&ContainerStopOpts::default()).await;
            let _ = container.delete().await;
        }

        multi.stop();
    }

    // Print the correct outro message based on whether the build was cancelled.
    if ctx.cancellation_token.is_cancelled() || matches!(last_result, ActionResult::Cancelled) {
        outro_cancel("Build cancelled")?;
    } else if let ActionResult::Failed(_, build_log) = last_result {
        outro_note(
            "Build failed".bold().red(),
            format!(
                "{reset_color}The {env_name} environment build failed.",
                reset_color = style(""),
                env_name = env_name.bold().magenta()
            ),
        )?;
        cliclack::log::error(build_log.join("\n"))?;
    } else {
        outro_note(
            "Finished!".bold().green(),
            format!(
                "{reset_color}The {env_name} environment has been built successfully.",
                reset_color = style(""),
                env_name = env_name.bold().magenta()
            ),
        )?;
    }

    Ok(())
}

/// Registers a shutdown handler to stop and remove the build container if
/// the user cancels the build using Ctrl+C.
async fn register_shutdown(ctx: &CliContext) {
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
            let _ = container.delete().await;
        }
    })
    .await;
}

/// Kills the existing build container if it exists.
async fn kill_existing_build_container(ctx: &CliContext) -> Result<()> {
    if let Some((container_id, _)) = ctx
        .docker()
        .find_container_by_name("stackify-build")
        .await?
    {
        let container = ctx.docker().api().containers().get(container_id);
        let _ = container.stop(&ContainerStopOpts::default()).await;
        let _ = container.delete().await;
    }

    Ok(())
}

/// Creates a new build container for the given service.
async fn create_build_container(
    ctx: &CliContext,
    service: &EnvironmentService,
) -> Result<Container> {
    let mut env_vars = HashMap::<String, String>::new();
    if ServiceType::StacksMiner.is(service.service_type.id) {
        let target = service
            .version
            .git_target
            .as_ref()
            .map(|x| x.target.clone())
            .unwrap_or_default();
        env_vars.insert("BUILD_STACKS".into(), target);
    }

    let create_opts = ContainerCreateOpts::for_stackify_build_container(
        ctx.docker().user(),
        &ctx.host_dirs,
        env_vars,
    );

    let build = ctx.docker().api().containers().create(&create_opts).await?;

    Ok(build)
}

async fn follow_build_logs(
    ctx: &CliContext,
    build: &Container,
    spinner: &ProgressBar,
) -> Result<ActionResult> {
    let mut log_stream = build.logs(
        &LogsOpts::builder()
            .stderr(true)
            .stdout(true)
            .follow(true)
            .all()
            .build(),
    );

    let mut accumulated_log = vec![];

    let building_text = style("Building...");

    while let Some(log) = log_stream.next().await {
        if ctx.cancellation_token.is_cancelled() {
            spinner.cancel("Build cancelled");
            kill_existing_build_container(ctx).await?;
            return Ok(ActionResult::Cancelled);
        }
        match log {
            Ok(TtyChunk::StdOut(chunk)) => {
                let msg = String::from_utf8_lossy(&chunk);
                let msg = msg.strip_suffix("\n").unwrap_or(&msg).trim();
                if msg.is_empty() {
                    continue;
                }
                spinner.set_message(format!("\u{8}{building_text} {msg}", msg = msg.dimmed()));
            }
            Ok(TtyChunk::StdErr(chunk)) => {
                let msg = String::from_utf8_lossy(&chunk);
                let msg = msg.strip_suffix("\n").unwrap_or(&msg).trim();
                if msg.is_empty() {
                    continue;
                }
                accumulated_log.push(msg.to_string());
                spinner.set_message(format!("\u{8}{building_text} {msg}", msg = msg.dimmed()));
            }
            Ok(TtyChunk::StdIn(_)) => unreachable!(),
            Err(e) => {
                spinner.set_message(format!(
                    "\u{8}{building_text} {msg}",
                    msg = e.to_string().dimmed()
                ));
            }
        }
    }

    let container_result = build.wait().await?;
    if container_result.status_code != 0 {
        spinner.stop(format!(
            "{} Build failed with status code {}",
            "⨯".red(),
            container_result.status_code
        ));
        return Ok(ActionResult::Failed(
            container_result.status_code,
            accumulated_log,
        ));
    } else {
        spinner.stop(format!("{} {}", "✔".green(), "Build complete"));
    }

    Ok(ActionResult::Success)
}
