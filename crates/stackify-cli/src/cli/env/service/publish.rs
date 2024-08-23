use clap::Args;
use color_eyre::Result;
use stackify_common::types::EnvironmentName;

use crate::cli::context::CliContext;

#[derive(Debug, Args)]
pub struct ServicePublishArgs {
    /// The name of the service to publish.
    #[arg(
        required = false, 
        value_name = "SERVICE",
        short = 's',
        long = "service"
    )]
    pub svc_name: Option<String>,

    /// The name of the environment to which the service belongs. You can omit
    /// this argument if the service is unique across all environments, otherwise
    /// you will receive an error.
    #[arg(
        required = false,
        value_name = "ENVIRONMENT",
        short = 'e',
        long = "env"
    )]
    pub env_name: Option<String>,

    /// The port on the host which the service should be published (exposed) on.
    /// This must be a valid port number between 1 and 65535 and not already in use.
    #[arg(
        required = false,
        value_name = "PORT",
        short = 'p',
        long = "port"
    )]
    pub port: Option<u16>,
}

pub async fn exec(ctx: &CliContext, args: ServicePublishArgs) -> Result<()> {
    let env_name = if let Some(name) = args.env_name {
        EnvironmentName::new(&name)?;
    } else {
        todo!("prompt for environment");
    };

    todo!()
}