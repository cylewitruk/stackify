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
    /// Add a new service version to the available service versions.
    #[clap(visible_aliases = ["add", "new"])]
    AddVersion(AddServiceVersionArgs),
    /// Remove a service version from the available service versions. This
    /// command will fail if the calculated service + version name is already
    /// in use.
    #[clap(visible_alias = "rm")]
    RemoveVersion(RemoveServiceVersionArgs),
    /// List all available services and their versions, plus their detailed
    /// information.
    #[clap(visible_alias = "ls")]
    List,
    /// Display detailed information about a service and its versions.
    #[clap(visible_alias = "insp")]
    Inspect
}

#[derive(Debug, Args)]
pub struct AddServiceVersionArgs {
    #[arg(
        short = 's',
        long = "service",
        visible_alias = "svc",
        required = true,
        value_name = "SERVICE"
    )]
    pub svc_name: String,

    /// The version of the service to add, for example: `0.21.0`, '
    #[arg(
        required = true,
        value_name = "NAME"
    )]
    pub name: String,

}

#[derive(Debug, Args)]
pub struct RemoveServiceVersionArgs {
    #[arg(
        required = true,
        value_name = "SERVICE"
    )]
    pub svc_name: String
}

#[derive(Debug, Args)]
pub struct ImportArgs {
    #[arg(
        short = 'f',
        long,
        required = true
    )]
    pub file: String,
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
