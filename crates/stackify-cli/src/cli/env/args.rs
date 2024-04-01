use clap::{Args, Subcommand};

use super::service::{add::ServiceAddArgs, config::ServiceConfigArgs};

#[derive(Debug, Args)]
pub struct EnvArgs {
    #[command(subcommand)]
    pub commands: EnvSubCommands,
}

#[derive(Debug, Subcommand)]
pub enum EnvSubCommands {
    /// Displays a list of created environments.
    #[clap(visible_alias = "ls")]
    List(ListArgs),
    /// Create a new environment.
    #[clap(visible_aliases = ["new", "add"])]
    Create(CreateArgs),
    /// Builds the specified environment, compiling the necessary binaries for
    /// the services if needed and creating the Docker containers which will be
    /// used for runtime. The environment will not be started, however.
    Build(BuildArgs),
    /// Displays detailed information about the specified environment.
    Inspect(InspectArgs),
    /// Removes the specified environment and all associated resources. This
    /// action is irreversible.
    #[clap(visible_alias = "rm")]
    Delete(DeleteArgs),
    /// Starts the specified environment using its current configuration. If the
    /// environment has not yet been built, it will be built first, which may
    /// take some time. If the environment is already running, this command will
    /// have no effect.
    #[clap(visible_alias = "up")]
    Start(StartArgs),
    /// Stops the specified environment.
    Stop(StopArgs),
    /// Stops the specified environment if it is running and removes all
    /// associated resources, without actually deleting the environment.
    Down(DownArgs),
    /// Commands for managing environments' services.
    #[clap(visible_aliases = ["svc"])]
    Service(ServiceArgs),
    /// Commands for managing environments' epoch configurations.
    Epoch(EpochArgs),
    /// Update the environment's configuration. Note that changes made here will
    /// not affect the environment until it is restarted.
    Set(SetArgs),
}

#[derive(Debug, Args)]
pub struct SetArgs {
    /// The name of the environment.
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
pub struct EpochArgs {
    #[command(subcommand)]
    pub commands: EpochSubCommands,
}

#[derive(Debug, Subcommand)]
pub enum EpochSubCommands {
    /// Prints the current epoch-map for the specified environment.
    List(EpochListArgs),
    /// Modify the epoch-map for the specified environment.
    Edit(EpochEditArgs),
}

#[derive(Debug, Args)]
pub struct EpochListArgs {
    /// The name of the environment to which the epoch-map belongs.
    #[arg(required = true, value_name = "ENVIRONMENT")]
    pub env_name: String,
}

#[derive(Debug, Args)]
pub struct EpochEditArgs {
    /// The name of the environment to which the epoch-map belongs.
    #[arg(required = true, value_name = "ENVIRONMENT")]
    pub env_name: String,
}

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
    #[arg(required = false, value_name = "ENVIRONMENT")]
    pub env_name: Option<String>,
}

#[derive(Debug, Args)]
pub struct BuildArgs {
    #[arg(required = true, value_name = "ENVIRONMENT")]
    pub env_name: String,
}

#[derive(Debug, Args)]
pub struct DownArgs {
    #[arg(required = true, value_name = "ENVIRONMENT")]
    pub env_name: String,
}

#[derive(Debug, Args)]
pub struct InspectArgs {
    /// The name of the environment to inspect.
    #[arg(required = true, value_name = "ENVIRONMENT")]
    pub env_name: String,
}

#[derive(Debug, Args)]
pub struct ListArgs {}

#[derive(Debug, Subcommand)]
pub enum ListSubCommands {
    Environments,
    Services,
}

#[derive(Debug, Args)]
pub struct CreateArgs {
    /// The name of the environment to create.
    #[arg(required = true, value_name = "NAME")]
    pub env_name: String,
    /// The speed at which blocks are mined in the Bitcoin network.
    #[arg(
        required = false,
        short,
        long,
        default_value = "30",
        value_name = "SECONDS"
    )]
    pub bitcoin_block_speed: u32,
}

#[derive(Debug, Args)]
pub struct DeleteArgs {
    #[arg(required = true, value_name = "NAME")]
    pub env_name: String,
}

#[derive(Debug, Args)]
pub struct StartArgs {
    #[arg(required = true, value_name = "NAME")]
    pub env_name: String,
}

#[derive(Debug, Args)]
pub struct StopArgs {
    #[arg(required = true, value_name = "NAME")]
    pub env_name: String,
}
