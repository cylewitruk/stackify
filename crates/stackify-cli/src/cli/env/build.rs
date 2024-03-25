use color_eyre::Result;
use console::style;
use stackify_common::EnvironmentName;

use crate::{cli::ERROR, context::CliContext};

use super::args::BuildArgs;

pub fn exec(ctx: &CliContext, args: BuildArgs) -> Result<()> {
    let env_name = EnvironmentName::new(&args.env_name)?;

    // Check if the environment has any services defined. If not, return an error.
    let env = ctx.db.list_environment_services(env_name.as_ref())?;
    if env.is_empty() {
        println!(
            "{} The '{}' environment has no services defined, so there is nothing to start.\n",
            *ERROR, env_name
        );
        println!("Please define at least one service before starting the environment.");
        println!(
            "See the {} command for more information.",
            style("stackify env service").white().bold()
        );
        return Ok(());
    }

    Ok(())
}
