use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub enum ConfigSubCommands {
    /// This will completely remove all Stackify resources from the system.
    Reset,
    /// Import a Stackify export file.
    Import(ImportArgs),
    /// Export Stackify configuration, which can be imported later. This is
    /// useful for sharing configurations between different machines.
    Export(ExportArgs),
    /// Commands for working with the services (i.e. Bitcoin nodes, Stacks nodes, etc.)
    /// and their configurations.
    Services(ServicesArgs),
}

#[derive(Debug, Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub commands: ConfigSubCommands,
}

#[derive(Debug, Args)]
pub struct ServicesArgs {
    #[command(subcommand)]
    pub subcommands: ServiceSubCommands,
}

#[derive(Debug, Subcommand)]
pub enum ServiceSubCommands {
    /// Add a new service to the configuration.
    Add,
    /// Remove a service from the configuration.
    Remove,
    /// List all services in the configuration.
    List,
    /// Display detailed information about a service.
    Inspect
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
