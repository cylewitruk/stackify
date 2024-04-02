use cliclack::{intro, outro};
use color_eyre::eyre::eyre;
use color_eyre::Result;
use console::style;
use stackify_common::types::EnvironmentName;

use crate::cli::theme::ThemedObject;
use crate::cli::{context::CliContext, warn};
use crate::db::cli_db::CliDatabase as _;

use super::args::BuildArgs;

pub fn exec(ctx: &CliContext, args: BuildArgs) -> Result<()> {
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

    let build_container = match ctx.docker.find_container_by_name("/stackify-build")? {
        Some(container) => {
            cliclack::log::info(format!("Found existing Stackify build container."))?;
            container
        }
        None => {
            // cliclack::log::info(format!("Creating Stackify build container."));
            // let container = ctx.docker.create_stackify_build_container(&env.bin_dir, &env.entrypoint)?;
            // Some(container)
            cliclack::log::error(format!("The Stackify build container has not yet been created, have you run `stackify init`?"))?;
            outro("Build cancelled.".red())?;
            return Ok(());
        }
    };

    cliclack::log::info(format!("Starting Stackify build container."))?;
    ctx.docker.start_build_container()?;
    cliclack::log::info(format!("Stackify build container started."))?;

    outro("Finished building the environment.")?;

    Ok(())
}
