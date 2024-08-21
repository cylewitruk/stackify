use clap::{Args, Subcommand};
use cliclack::intro;
use color_eyre::Result;
use stackify_common::types::EnvironmentName;

use crate::{cli::{context::CliContext, theme::ThemedObject}, db::cli_db::CliDatabase};

#[derive(Debug, Args)]
pub struct ContractArgs {
    #[command(subcommand)]
    pub commands: ContractSubCommands,
}

#[derive(Debug, Subcommand)]
pub enum ContractSubCommands {
    Deploy(ContractDeployArgs),
    Call(ContractCallArgs),
}

#[derive(Debug, Args)]
pub struct ContractDeployArgs {
    /// The name of the environment in which to deploy the contract.
    #[arg(required = true, value_name = "ENVIRONMENT")]
    pub env_name: String
}

#[derive(Debug, Args)]
pub struct ContractCallArgs {
    /// The name of the environment in which the contract is deployed.
    #[arg(required = true, value_name = "ENVIRONMENT")]
    pub env_name: String
}

pub async fn exec(ctx: &CliContext, args: ContractArgs) -> Result<()> {
    match args.commands {
        ContractSubCommands::Deploy(inner_args) => {
            exec_deploy(ctx, inner_args).await
        },
        ContractSubCommands::Call(inner_args) => {
            exec_call(ctx, inner_args).await
        }
    }
}

async fn exec_deploy(ctx: &CliContext, args: ContractDeployArgs) -> Result<()> {
    let db = ctx.db.as_clidb();
    let env_name = EnvironmentName::new(&args.env_name)?;
    let env = db.load_environment(&env_name)?;

    intro(format!("{}", "Deploy Contract".bold()))?;
    
    todo!()
}

async fn exec_call(ctx: &CliContext, args: ContractCallArgs) -> Result<()> {
    todo!()
}