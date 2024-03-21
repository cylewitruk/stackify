use std::fmt::Debug;

use clap::Args;
use color_eyre::eyre::Result;
use console::style;

use crate::context::CliContext;

#[derive(Debug, Args)]
pub struct InfoArgs {
    #[arg(
        short = 'd',
        long,
        default_value = "false",
        required = false,
    )]
    docker: bool,

    #[arg(
        short = 'e', 
        long,
        default_value = "false",
        required = false,
    )]
    epochs: bool,

    #[arg(
        short = 's',
        long,
        default_value = "false",
        required = false,
    )]
    service_types: bool
}

pub fn exec(ctx: &CliContext, args: InfoArgs) -> Result<()> {

    println!("workspace root: {:?}", std::env::var("CARGO_MANIFEST_DIR"));

    println!("{}", style("Stackify Status:").bold());
    println!(" ‣ Environments: {}", ctx.db.list_environments()?.len());

    if args.docker {
        exec_display_docker_info(ctx)?;
    }

    if args.epochs {
        exec_list_epochs(ctx)?;
    }
    
    if args.service_types {
        exec_list_service_types(ctx)?;
    }

    Ok(())
}

fn exec_display_docker_info(ctx: &CliContext) -> Result<()> {
    let docker_version = ctx.docker.get_docker_version()?;

    println!("{}", style("Docker Information").bold());
    println!(" ‣ Version: {}", docker_version.version);
    println!(" ‣ API Version: {}", docker_version.api_version);
    println!(" ‣ Min API Version: {}", docker_version.min_api_version);
    println!("");

    Ok(())
}

fn exec_list_service_types(ctx: &CliContext) -> Result<()> {
    let service_types = ctx.db.list_service_types()?;

    println!("{}", style("The following service types are available:").bold());
    for service_type in service_types {
        println!("  ‣ {}", service_type.name);
    }
    Ok(())
}

fn exec_list_epochs(_ctx: &CliContext) -> Result<()> {
    println!("List epochs");
    Ok(())
}