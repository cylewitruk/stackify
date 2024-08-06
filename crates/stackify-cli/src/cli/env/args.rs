use clap::{Args, Subcommand};

use super::{
    epoch::EpochArgs, 
    keychain::KeychainArgs, 
    service::ServiceArgs
};

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
    #[clap(visible_aliases = ["create", "add"])]
    New(NewArgs),
    /// Builds the specified environment, compiling the necessary binaries for
    /// the services if needed and creating the Docker containers which will be
    /// used for runtime. The environment will not be started, however.
    Build(BuildArgs),
    /// Displays detailed information about the specified environment.
    Inspect(InspectArgs),
    /// Removes the specified environment and all associated resources. This
    /// action is irreversible.
    #[clap(visible_alias = "rm")]
    Remove(RemoveArgs),
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
    /// Manage services for the specified environment.
    #[clap(visible_aliases = ["svc"])]
    Service(ServiceArgs),
    /// Manage the epoch-map for the specified environment.
    Epoch(EpochArgs),
    /// Update the environment's configuration. Note that changes made here will
    /// not affect the environment until it is restarted.
    Set(SetArgs),
    /// Manage keychains (Stacks accounts) for the environment.
    Keychain(KeychainArgs),
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
pub struct NewArgs {
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
pub struct RemoveArgs {
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
