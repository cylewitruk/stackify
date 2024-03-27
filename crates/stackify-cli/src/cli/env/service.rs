
use color_eyre::Result;
use stackify_common::EnvironmentName;

use super::CliContext;

use super::args::{ServiceArgs, ServiceInspectArgs, ServiceListArgs, ServiceRemoveArgs,
    ServiceSubCommands,
};

pub mod add;
pub mod config;

pub fn exec_service(ctx: &CliContext, args: ServiceArgs) -> Result<()> {
    match args.commands {
        ServiceSubCommands::Add(inner_args) => {
            add::exec(ctx, inner_args)?;
        }
        ServiceSubCommands::Remove(inner_args) => {
            exec_remove(ctx, inner_args)?;
        }
        ServiceSubCommands::Inspect(inner_args) => {
            exec_inspect(ctx, inner_args)?;
        }
        ServiceSubCommands::List(inner_args) => {
            exec_list(ctx, inner_args)?;
        }
        ServiceSubCommands::Config => {
            config::exec(ctx)?;
        }
    }
    Ok(())
}

fn exec_remove(_ctx: &CliContext, args: ServiceRemoveArgs) -> Result<()> {
    let _env_name = EnvironmentName::new(&args.env_name)?;
    todo!()
}

fn exec_inspect(_ctx: &CliContext, args: ServiceInspectArgs) -> Result<()> {
    let _env_name = EnvironmentName::new(&args.env_name)?;
    todo!()
}

fn exec_list(_ctx: &CliContext, args: ServiceListArgs) -> Result<()> {
    let _env_name = EnvironmentName::new(&args.env_name)?;
    todo!()
}