use clap::Args;
use color_eyre::eyre::Result;

use crate::context::CliContext;

#[derive(Debug, Args)]
pub struct CleanArgs {}

pub fn exec(ctx: &CliContext, _args: CleanArgs) -> Result<()> {
    let networks = ctx.docker.list_stacks_networks()?;
    for network in networks {
        println!("{:?}", network);
        ctx.docker.rm_network(&network.name)?;
    }
    Ok(())
}