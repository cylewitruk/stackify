use color_eyre::Result;
use stackify_common::types::EnvironmentName;

use super::CliContext;

use super::args::{ServiceArgs, ServiceInspectArgs, ServiceListArgs, ServiceSubCommands};

pub mod add;
pub mod config;
pub mod list;
pub mod remove;

pub fn exec_service(ctx: &CliContext, args: ServiceArgs) -> Result<()> {
    match args.commands {
        ServiceSubCommands::Add(inner_args) => {
            add::exec(ctx, inner_args)?;
        }
        ServiceSubCommands::Remove(inner_args) => {
            remove::exec(ctx, inner_args)?;
        }
        ServiceSubCommands::Inspect(inner_args) => {
            exec_inspect(ctx, inner_args)?;
        }
        ServiceSubCommands::List(inner_args) => {
            list::exec(ctx, inner_args)?;
        }
        ServiceSubCommands::Config(inner_args) => {
            config::exec(ctx, inner_args)?;
        }
    }
    Ok(())
}

fn exec_inspect(_ctx: &CliContext, args: ServiceInspectArgs) -> Result<()> {
    let _env_name = EnvironmentName::new(&args.env_name)?;
    todo!()
}
