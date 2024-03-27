use color_eyre::Result;
use clap::Args;

use crate::cli::context::CliContext;

#[derive(Debug, Args)]
pub struct ServiceConfigArgs {
    /// The name of the service of which to inspect.
    #[arg(required = true, value_name = "NAME")]
    pub svc_name: String,

    /// The name of the environment to which the service belongs. You can omit
    /// this argument if the service is unique across all environments, otherwise
    /// you will receive an error.
    #[arg(
        required = false,
        value_name = "NAME",
        short = 'e',
        long = "environment",
        visible_alias = "env"
    )]
    pub env_name: String,
}

pub fn exec(ctx: &CliContext) -> Result<()> {
    let editor = scrawl::with(&"Hello, world!");
    let output = editor
        .expect("failed to open default editor");
    let text = output.to_string()
        .expect("failed to convert output to string");
    cliclack::log::remark(text)?;
    Ok(())
}