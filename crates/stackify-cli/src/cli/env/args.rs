use clap::{Args, Subcommand};

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
    Service(ServiceArgs)
}

#[derive(Debug, Args)]
pub struct ServiceArgs {
    #[command(subcommand)]
    pub commands: ServiceSubCommands,
}

#[derive(Debug, Subcommand)]
pub enum ServiceSubCommands {
    Add(ServiceAddArgs),
    #[clap(visible_aliases = ["rm", "del"])]
    Remove(ServiceRemoveArgs),
    Inspect(ServiceInspectArgs),
    List(ServiceListArgs)
}

#[derive(Debug, Args)]
pub struct ServiceInspectArgs {
    /// The name of the service of which to inspect.
    #[arg(
        required = true,
        value_name = "NAME"
    )]
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
    pub env_name: String
}

#[derive(Debug, Args)]
pub struct ServiceAddArgs {
    /// The name of the service to add. For more information on the available
    /// services, see the `stackify config services` command.
    #[arg(
        required = true,
        value_name = "NAME"
    )]
    pub svc_name: String,

    /// Indicates whether or not an interactive prompt should be used for providing
    /// the required information for this command (recommended!). This flag is
    /// set by default.
    #[arg(
        required = false,
        short = 'i',
        default_value = "true"
    )]
    pub interactive: bool,

    /// The name of the environment to which the service should be added.
    #[arg(
        required = true,
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
    pub env_name: String
}

#[derive(Debug, Args)]
pub struct ServiceListArgs {
    /// The name of the environment to list services for.
    #[arg(
        required = true,
        value_name = "NAME"
    )]
    pub env_name: String,
}

#[derive(Debug, Args)]
pub struct BuildArgs {
    #[arg(
        required = true, 
        value_name = "NAME"
    )]
    pub env_name: String,
}

#[derive(Debug, Args)]
pub struct DownArgs {
    #[arg(
        required = true, 
        value_name = "NAME"
    )]
    pub env_name: String,
}

#[derive(Debug, Args)]
pub struct InspectArgs {
    /// The name of the environment to inspect.
    #[arg(
        required = true,
        value_name = "NAME"
    )]
    pub env_name: String,
}

#[derive(Debug, Args)]
pub struct ListArgs {
}

#[derive(Debug, Subcommand)]
pub enum ListSubCommands {
    Environments,
    Services,
}

#[derive(Debug, Args)]
pub struct CreateArgs {
    /// The name of the environment to create.
    #[arg(required = true, value_name = "NAME")]
    pub env_name : String,
    /// The speed at which blocks are mined in the Bitcoin network.
    #[arg(required = false, short, long, default_value = "30", value_name = "SECONDS")]
    pub bitcoin_block_speed: u32 
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
