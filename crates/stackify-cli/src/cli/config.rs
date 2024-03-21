use clap::{Args, Subcommand};
use color_eyre::eyre::Result;

use crate::context::CliContext;

#[derive(Debug, Subcommand)]
pub enum ConfigSubCommands {
    /// This will completely remove all stackify resources from the system.
    Reset
}

#[derive(Debug, Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub commands: ConfigSubCommands,
}

pub fn exec(ctx: &CliContext, args: ConfigArgs) -> Result<()> {
    match args.commands {
        ConfigSubCommands::Reset => exec_reset(ctx),
    }
    
}

fn exec_reset(ctx: &CliContext) -> Result<()> {
    if ctx.config_dir.ends_with(".stackify") {
        std::fs::remove_dir_all(ctx.config_dir.clone()).unwrap();
    }
    Ok(())
}