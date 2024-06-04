use cliclack::{intro, multi_progress, outro_cancel, outro_note, ProgressBar};
use color_eyre::Result;
use console::style;
use docker_api::conn::TtyChunk;
use docker_api::models::ContainerState;
use docker_api::opts::{ContainerCreateOpts, ContainerStopOpts, LogsOpts};
use docker_api::Container;
use futures_util::StreamExt;
use regex::Regex;
use stackify_common::types::{EnvironmentName, EnvironmentService};
use stackify_common::ServiceType;
use std::collections::HashMap;

use crate::cli::context::CliContext;
use crate::cli::log::clilog;
use crate::cli::theme::ThemedObject;
use crate::db::cli_db::CliDatabase as _;
use crate::docker::opts::CreateContainer;
use crate::docker::ActionResult;

use super::args::BuildArgs;

pub async fn exec(ctx: &CliContext, args: BuildArgs) -> Result<()> {
    let db = ctx.db.as_clidb();
    let env_name = EnvironmentName::new(&args.env_name)?;
    let env = db.load_environment(&env_name)?;

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

    let mut last_result = ActionResult::Success("".to_string());

    // Iterate through each service in the environment and build them.
    for service in env.services.iter() {
        clilog!(
            "Building service: {} ({})",
            service.name,
            service.version.version
        );
        if ![
            ServiceType::StacksMiner,
            ServiceType::StacksFollower,
            ServiceType::StacksSigner,
        ]
        .contains(&ServiceType::from_i32(service.service_type.id)?)
        {
            clilog!(
                "Skipping service: {} ({})",
                service.name,
                service.service_type.name
            );
            cliclack::log::step(format!(
                "Skipping {service_name} {service_type} {reason}",
                service_name = service.name.cyan(),
                service_type = format!("[{}] »", service.service_type.name).dimmed(),
                reason = "nothing to build".dimmed()
            ))?;
            continue;
        }

        let multi = multi_progress(format!(
            "Building {} version {} {}",
            service.name.cyan(),
            service.version.version.magenta(),
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
        if !matches!(last_result, ActionResult::Success(_)) {
            break;
        }

        // Stop & remove the build container, ignoring errors
        if !ctx.cancellation_token.is_cancelled() {
            let _ = container.stop(&ContainerStopOpts::default()).await;
        } else {
            let _ = container
                .stop(&ContainerStopOpts::builder().signal("SIGKILL").build())
                .await;
        };

        clilog!("waiting for build container to stop...");
        let _ = container.wait().await;
        let _ = container.delete().await;

        if let ActionResult::Success(commit_hash) = &last_result {
            ctx.db
                .update_service_version_build_details(service.version.id, Some(&commit_hash))?;
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
        if let Some((container_id, _)) = d
            .find_container_by_name(&EnvironmentName::new("stackify-build").unwrap())
            .await
            .unwrap()
        {
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
    clilog!("checking for existing build container...");
    if let Some((container_id, _)) = ctx
        .docker()
        .find_container_by_name(&EnvironmentName::new("stackify-build")?)
        .await?
    {
        clilog!("found existing build container, stopping and removing...");
        let container = ctx.docker().api().containers().get(container_id.clone());
        let _ = container.stop(&ContainerStopOpts::default()).await;
        let _ = container.wait().await;
        let _ = container.delete().await;
        while ctx
            .docker()
            .api()
            .containers()
            .get(container_id.clone())
            .inspect()
            .await
            .is_ok()
        {
            clilog!("waiting for build container to stop...");
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    Ok(())
}

/// Creates a new build container for the given service.
async fn create_build_container(
    ctx: &CliContext,
    service: &EnvironmentService,
) -> Result<Container> {
    kill_existing_build_container(ctx).await?;

    let mut env_vars = HashMap::<String, String>::new();
    if [
        ServiceType::StacksMiner,
        ServiceType::StacksFollower,
    ]
    .contains(&ServiceType::from_i32(service.service_type.id)?)
    {
        let target = service
            .version
            .git_target
            .as_ref()
            .map(|x| x.target.clone())
            .unwrap_or_default();

        env_vars.insert("BUILD_STACKS".into(), target);
    } else if [
        ServiceType::StacksSigner,
    ]
    .contains(&ServiceType::from_i32(service.service_type.id)?)
    {
        let target = service
            .version
            .git_target
            .as_ref()
            .map(|x| x.target.clone())
            .unwrap_or_default();

        env_vars.insert("BUILD_SIGNER".into(), target);
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
    let commit_hash_regex = Regex::new("^COMMIT_HASH=([a-z0-9]+)$")?;
    let mut commit_hash = None;

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
                clilog!("STDOUT: {}", msg);

                if msg.is_empty() {
                    continue;
                }
                
                spinner.set_message(format!("{building_text} {msg}", msg = msg.dimmed()));

                commit_hash_regex.captures(&msg).map(|captures| {
                    clilog!("Commit hash: {}", captures.get(1).unwrap().as_str());
                    commit_hash = Some(captures.get(1).unwrap().as_str().to_string());
                });
            }
            Ok(TtyChunk::StdErr(chunk)) => {
                let msg = String::from_utf8_lossy(&chunk);
                let msg = msg.strip_suffix("\n").unwrap_or(&msg).trim();
                //clilog!("STDERR: {}", msg);

                if msg.is_empty() {
                    continue;
                }
                accumulated_log.push(msg.to_string());
                spinner.set_message(format!("{building_text} {msg}", msg = msg.dimmed()));
            }
            Ok(TtyChunk::StdIn(_)) => unreachable!(),
            Err(e) => {
                clilog!("ERR: {}", e.to_string());
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
        spinner.stop(format!(
            "{} {} {}",
            "✔".green(),
            "Build complete",
            format!(
                "({})",
                commit_hash.as_ref().map(|x| x.as_str()).unwrap_or_default()
            )
            .dimmed()
        ));
    }

    Ok(ActionResult::Success(
        commit_hash.unwrap_or_default().to_string(),
    ))
}
