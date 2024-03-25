use color_eyre::Result;

use crate::context::CliContext;

use self::{args::{ConfigArgs, ConfigSubCommands}, services::exec_services};

pub mod args;
pub mod services;
pub mod epochs;

pub fn exec(ctx: &CliContext, args: ConfigArgs) -> Result<()> {
    match args.commands {
        ConfigSubCommands::Reset => exec_reset(ctx),
        ConfigSubCommands::Import(_) => todo!(),
        ConfigSubCommands::Export(_) => todo!(),
        ConfigSubCommands::Services(inner_args) => exec_services(ctx, inner_args),
        ConfigSubCommands::Epochs(inner_args) => epochs::exec(ctx, inner_args),
    }
}

fn exec_reset(ctx: &CliContext) -> Result<()> {
    if ctx.config_dir.ends_with(".stackify") {
        std::fs::remove_dir_all(ctx.config_dir.clone()).unwrap();
    }
    Ok(())
}