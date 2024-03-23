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
    #[clap(visible_alias = "new")]
    Create(CreateArgs),
    /// Displays detailed information about the specified environment.
    Inspect(InspectArgs),
    /// Removes the specified environment and all associated resources. This
    /// action is irreversible.
    #[clap(visible_alias = "rm")]
    Delete(DeleteArgs),
    /// Starts the specified environment using its current configuration.
    Start(StartArgs),
    /// Stops the specified environment.
    Stop(StopArgs),
    /// Stops the specified environment if it is running and removes all
    /// associated resources, without actually deleting the environment.
    Down(DownArgs),
}

#[derive(Debug, Args)]
pub struct DownArgs {
    #[arg(required = true)]
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
    #[arg(required = true)]
    pub env_name : Option<String>,
    /// The speed at which blocks are mined in the Bitcoin network.
    #[arg(required = false, short, long, default_value = "30", value_name = "SECONDS")]
    pub bitcoin_block_speed: u32 
}

#[derive(Debug, Args)]
pub struct DeleteArgs {}

#[derive(Debug, Args)]
pub struct StartArgs {
    #[arg(required = true)]
    pub env_name: String,
}

#[derive(Debug, Args)]
pub struct StopArgs {
    #[arg(required = true)]
    pub env_name: String,
}
