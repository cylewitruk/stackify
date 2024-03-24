use color_eyre::Result;
use stackify_common::EnvironmentName;

use crate::context::CliContext;

use super::args::{ServiceAddArgs, ServiceArgs, ServiceInspectArgs, ServiceListArgs, ServiceRemoveArgs, ServiceSubCommands};

pub fn exec_service(ctx: &CliContext, args: ServiceArgs) -> Result<()> {
    match args.commands {
        ServiceSubCommands::Add(inner_args) => {
            exec_add(ctx, inner_args)?;
        }
        ServiceSubCommands::Remove(inner_args) => {
            exec_remove(ctx, inner_args)?;
        }
        ServiceSubCommands::Inspect(inner_args) => {
            exec_inspect(ctx, inner_args)?;
        },
        ServiceSubCommands::List(inner_args) => {
            exec_list(ctx, inner_args)?;
        }
    }
    Ok(())
}

fn exec_add(_ctx: &CliContext, args: ServiceAddArgs) -> Result<()> {
    let env_name = EnvironmentName::new(&args.env_name)?;
    Ok(())
}

fn exec_remove(_ctx: &CliContext, args: ServiceRemoveArgs) -> Result<()> {
    let env_name = EnvironmentName::new(&args.env_name)?;
    Ok(())
}

fn exec_inspect(_ctx: &CliContext, args: ServiceInspectArgs) -> Result<()> {
    let env_name = EnvironmentName::new(&args.env_name)?;
    Ok(())
}

fn exec_list(_ctx: &CliContext, args: ServiceListArgs) -> Result<()> {
    let env_name = EnvironmentName::new(&args.env_name)?;
    Ok(())
}