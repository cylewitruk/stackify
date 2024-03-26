use color_eyre::eyre::{bail, eyre};
use color_eyre::Result;
use inquire::ui::RenderConfig;
use inquire::validator::Validation;
use inquire::{Confirm, Select, Text};
use stackify_common::EnvironmentName;

use crate::cli::theme::{inquire_style, theme, Inquire, ThemedObject};
use crate::cli::{info, warn};
use crate::db::model::Epoch;
use crate::util::FilterByServiceType;

use super::CliContext;
use super::service_add;

use super::args::{
    ServiceAddArgs, ServiceArgs, ServiceInspectArgs, ServiceListArgs, ServiceRemoveArgs,
    ServiceSubCommands,
};

pub fn exec_service(ctx: &CliContext, args: ServiceArgs) -> Result<()> {
    match args.commands {
        ServiceSubCommands::Add(inner_args) => {
            service_add::exec(ctx, inner_args)?;
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
            exec_config(ctx)?;
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

fn exec_config(_ctx: &CliContext) -> Result<()> {
    todo!()
}