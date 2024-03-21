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
    Create(CreateArgs),
    /// Remove an environment.
    #[clap(visible_alias = "rm")]
    Delete(DeleteArgs),
    /// Starts an environment using its current configuration.
    Start(StartArgs),
    /// Stops a currently running environment.
    Stop(StopArgs),
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
    /// The name of the argument to create.
    #[arg(
        short = 'n',
        long,
        required = true,
    )]
    pub name: String,
    /// How quickly (in seconds) new Bitcoin blocks are mined in this environment.
    #[arg(
        short = 'b',
        long,
        default_value = "30"
    
    )]
    pub bitcoin_block_speed: u32
}

#[derive(Debug, Args)]
pub struct DeleteArgs {}

#[derive(Debug, Args)]
pub struct StartArgs {
    pub env_name: String,
}

#[derive(Debug, Args)]
pub struct StopArgs {}
