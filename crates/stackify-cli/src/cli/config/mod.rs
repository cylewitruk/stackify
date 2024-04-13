use color_eyre::Result;

use crate::cli::context::CliContext;

use self::{
    args::{ConfigArgs, ConfigSubCommands},
    services::exec_services,
};

pub mod args;
pub mod epochs;
pub mod services;

pub fn exec(ctx: &CliContext, args: ConfigArgs) -> Result<()> {
    match args.commands {
        ConfigSubCommands::Import(_) => todo!(),
        ConfigSubCommands::Export(_) => todo!(),
        ConfigSubCommands::Services(inner_args) => exec_services(ctx, inner_args),
        ConfigSubCommands::Epochs(inner_args) => epochs::exec(ctx, inner_args),
    }
}
