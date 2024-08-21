use clap::Args;
use color_eyre::eyre::{eyre, Result};

use crate::{
    cli::context::CliContext, docker::opts::ListNetworks, docker_api::opts::NetworkListOpts,
};

#[derive(Debug, Args)]
pub struct CleanArgs {}

pub async fn exec(ctx: &CliContext, _args: CleanArgs) -> Result<()> {
    let networks = ctx
        .docker()
        .api()
        .networks()
        .list(&NetworkListOpts::for_all_stackify_networks())
        .await?;

    for network in networks {
        println!("{:?}", network);
        let network_id = network.id.ok_or(eyre!("Network ID not found."))?;

        ctx.docker()
            .api()
            .networks()
            .get(network_id)
            .delete()
            .await?;
    }
    Ok(())
}
