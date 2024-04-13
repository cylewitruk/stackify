use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub enum ConfigSubCommands {
    /// Import a Stackify export file.
    Import(ImportArgs),
    /// Export Stackify configuration, which can be imported later. This is
    /// useful for sharing configurations between different machines.
    Export(ExportArgs),
    /// Commands for working with the services (i.e. Bitcoin nodes, Stacks nodes, etc.)
    /// and their configurations.
    Services(ServicesArgs),
    Epochs(EpochsArgs),
}

#[derive(Debug, Args)]
pub struct EpochsArgs {
    #[command(subcommand)]
    pub subcommands: EpochsSubCommands,
}

#[derive(Debug, Subcommand)]
pub enum EpochsSubCommands {
    /// Prints a list of all available epochs.
    #[clap(visible_alias = "ls")]
    List,
    /// Add a new epoch to the list of available epochs. This is considered an
    /// expert feature and should only be used if you know what you are doing.
    Add(AddEpochArgs),
    /// Remove an epoch from the available epochs. Note that an epoch cannot
    /// be removed if it is in use by a service version. To see the usages of
    /// an epoch, run `stackify config epochs inspect`.
    #[clap(visible_alias = "rm")]
    Remove,
    /// Display detailed information about an epoch, including its usages.
    #[clap(visible_aliases = ["insp", "show"])]
    Inspect,
}

#[derive(Debug, Args)]
pub struct AddEpochArgs {
    /// The name of the epoch to add. This must be a valid epoch name, for
    /// example: `2.05`, `2.4`, `3.0`, etc. The epoch must be unique, in decimal
    /// format, and greater than the current highest epoch.
    #[arg(required = true, value_name = "EPOCH", value_parser)]
    pub name: f32,

    /// Optionally specifies the default block height for this epoch. If not
    /// provided, the default block height will be set to the current highest
    /// default block height + 3.
    #[arg(required = false, value_name = "BLOCK_HEIGHT", long = "block-height")]
    pub block_height: Option<i32>,

    /// As adding a new epoch is considered an "expert feature", this flag is
    /// required to be set to confirm that the user understands the implications
    /// of adding a new epoch.
    #[arg(required = true, long = "force")]
    pub force: bool,
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
    /// Add a new version to one of the available service types.
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
    Inspect,
}

#[derive(Debug, Args)]
pub struct AddServiceVersionArgs {
    /// The version of the service to add, for example: `21.0`, 'PoX-5', etc.
    /// Note that different services types have different constraints regarding
    /// what can be used as a version.
    #[arg(required = true, value_name = "VERSION")]
    pub version: String,

    /// The minimum epoch that this service version is compatible with. This
    /// must be a valid epoch name, for example: `2.05`, `2.4`, `3.0`, etc. Note
    /// that the service will not be prevented from being used on a lower epoch
    /// as that may be your intent, but it will generate a warning.
    #[arg(required = false, value_name = "EPOCH", long = "min-epoch")]
    pub min_epoch: Option<String>,

    /// The maximum epoch that this service version is compatible with. This
    /// must be a valid epoch name, for example: `2.05`, `2.4`, `3.0`, etc. Note
    /// that the service will not be prevented from being used on a higher epoch
    /// as that may be your intent, but it will generate a warning. To view the
    /// available epochs, run `stackify config epochs list`.
    #[arg(required = false, value_name = "EPOCH", long = "max-epoch")]
    pub max_epoch: Option<String>,

    /// The git target for this service version. This can be a branch, tag, or
    /// commit hash. This is conditionally required/allowed based on the service
    /// type.
    /// Required for: ['stacks-miner', 'stacks-follower', 'stacks-signer']
    /// Not allowed for: ['bitcoin-miner', 'bitcoin-follower', 'stacks-stacker-self', 'stacks-stacker-pool']
    #[arg(
        required = false,
        value_name = "BRANCH|TAG|COMMIT",
        long = "git-target",
        help = GIT_TARGET_HELP,
        required_if_eq_any([
            ("version", "stacks-miner"),
            ("version", "stacks-follower"),
            ("version", "stacks-signer")
        ])
    )]
    pub git_target: Option<String>,
}

const GIT_TARGET_HELP: &str = r#"The git target for this service version. This can be a branch, tag, or commit hash. This is conditionally required/allowed based on the service type.

The prefix defines the type of git target: 'branch:', 'tag:', 'commit:'. For example, 'branch:main', 'tag:v1.0.0', 'commit:abcdef1234567890'.

Required for: [stacks-miner, stacks-follower, stacks-signer]
Not allowed for: [bitcoin-miner, bitcoin-follower, stacks-stacker-self, stacks-stacker-pool]"#;

#[derive(Debug, Args)]
pub struct RemoveServiceVersionArgs {
    #[arg(required = true, value_name = "SERVICE")]
    pub svc_name: String,
}

#[derive(Debug, Args)]
pub struct ImportArgs {
    #[arg(short = 'f', long, required = true)]
    pub file: String,
}

#[derive(Debug, Args)]
pub struct ExportArgs {
    #[arg(short, long, alias = "env", exclusive = true)]
    environment: String,
    #[arg(long, alias = "all-envs")]
    environments: bool,
    #[arg(short = 's', long = "services")]
    services: bool,
}
