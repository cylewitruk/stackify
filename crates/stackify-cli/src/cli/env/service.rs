use add::ServiceAddArgs;
use clap::{Args, Subcommand};
use color_eyre::Result;
use config::ServiceConfigArgs;
use stackify_common::types::EnvironmentName;

use super::CliContext;

pub mod add;
pub mod config;
pub mod list;
pub mod remove;

#[derive(Debug, Args)]
pub struct ServiceArgs {
    #[command(subcommand)]
    pub commands: ServiceSubCommands,
}

#[derive(Debug, Subcommand)]
pub enum ServiceSubCommands {
    /// Adds a new service to the specified environment.
    Add(ServiceAddArgs),
    #[clap(visible_aliases = ["rm", "del"])]
    /// Remove a service from the specified environment.
    Remove(ServiceRemoveArgs),
    #[clap(visible_aliases = ["insp", "show"])]
    /// Displays detailed information about the specified service.
    Inspect(ServiceInspectArgs),
    /// Displays a list of services for the specified environment.
    #[clap(visible_alias = "ls")]
    List(ServiceListArgs),
    /// Commands for managing service configuration files. This will provide you
    /// with an editor to manually edit the configuration file for the specified
    /// service.
    #[clap(visible_alias = "cfg")]
    Config(ServiceConfigArgs),
}

#[derive(Debug, Args)]
pub struct ServiceInspectArgs {
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

#[derive(Debug, Args)]
pub struct ServiceRemoveArgs {
    /// The name of the environment from which the service should be removed.
    /// You can omit this argument if the service is unique across all environments,
    /// otherwise you will receive an error.
    #[arg(
        required = false,
        value_name = "NAME",
        short = 'e',
        long = "environment",
        visible_alias = "env"
    )]
    pub env_name: String,
}

#[derive(Debug, Args)]
pub struct ServiceListArgs {
    /// The name of the environment to list services for.
    #[arg(required = true, value_name = "ENVIRONMENT")]
    pub env_name: String,
}

pub async fn exec_service(ctx: &CliContext, args: ServiceArgs) -> Result<()> {
    match args.commands {
        ServiceSubCommands::Add(inner_args) => add::exec(ctx, inner_args).await,
        ServiceSubCommands::Remove(inner_args) => remove::exec(ctx, inner_args),
        ServiceSubCommands::Inspect(inner_args) => exec_inspect(ctx, inner_args),
        ServiceSubCommands::List(inner_args) => list::exec(ctx, inner_args),
        ServiceSubCommands::Config(inner_args) => config::exec(ctx, inner_args),
    }
}

fn exec_inspect(_ctx: &CliContext, args: ServiceInspectArgs) -> Result<()> {
    let _env_name = EnvironmentName::new(&args.env_name)?;
    todo!()
}
