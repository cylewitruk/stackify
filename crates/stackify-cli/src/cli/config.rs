use clap::{Args, Subcommand};
use color_eyre::eyre::Result;

use crate::context::CliContext;

#[derive(Debug, Subcommand)]
pub enum ConfigSubCommands {
    /// This will completely remove all Stackify resources from the system.
    Reset,
    /// Import a Stackify export file.
    Import(ImportArgs),
    /// Export Stackify configuration, which can be imported later. This is
    /// useful for sharing configurations between different machines.
    Export(ExportArgs)
}

#[derive(Debug, Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub commands: ConfigSubCommands,
}

#[derive(Debug, Args)]
pub struct ImportArgs {
    #[arg(
        short = 'f',
        long,
        required = true
    )]
    file: String,
}

#[derive(Debug, Args)]
pub struct ExportArgs {
    #[arg(
        short,
        long,
        alias = "env",
        exclusive = true
    )]
    environment: String,
    #[arg(
        short,
        long,
        alias = "all-envs"
    )]
    environments: bool,
    #[arg(
        short = 's',
        long = "services"
    )]
    services: bool,
}

pub fn exec(ctx: &CliContext, args: ConfigArgs) -> Result<()> {
    match args.commands {
        ConfigSubCommands::Reset => exec_reset(ctx),
        ConfigSubCommands::Import(_) => todo!(),
        ConfigSubCommands::Export(_) => todo!(),
    }
    
}

fn exec_reset(ctx: &CliContext) -> Result<()> {
    if ctx.config_dir.ends_with(".stackify") {
        std::fs::remove_dir_all(ctx.config_dir.clone()).unwrap();
    }
    Ok(())
}