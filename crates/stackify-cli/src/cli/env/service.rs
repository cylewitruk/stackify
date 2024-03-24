use color_eyre::Result;

use crate::context::CliContext;

use super::args::ServiceArgs;

pub fn exec_service(_ctx: &CliContext, _args: ServiceArgs) -> Result<()> {
    println!("exec service");
    Ok(())
}