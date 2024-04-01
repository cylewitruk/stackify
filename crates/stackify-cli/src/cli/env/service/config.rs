use clap::Args;
use color_eyre::{eyre::eyre, Result};
use stackify_common::EnvironmentName;

use crate::{cli::context::CliContext, util::FindById};

#[derive(Debug, Args)]
pub struct ServiceConfigArgs {
    /// The name of the environment to which the service belongs. You can omit
    /// this argument if the service is unique across all environments, otherwise
    /// you will receive an error.
    #[arg(required = false, value_name = "ENVIRONMENT")]
    pub env_name: String,
}

pub fn exec(ctx: &CliContext, args: ServiceConfigArgs) -> Result<()> {
    let env_name = EnvironmentName::new(&args.env_name)?;
    let env = ctx.db.get_environment_by_name(env_name.as_ref())?;
    let service_types = ctx.db.list_service_types()?;
    let service_versions = ctx.db.list_service_versions()?;
    let service_type_files = ctx.db.list_service_type_files()?;
    let services = ctx.db.list_environment_services(env.id)?;

    let mut select_service = cliclack::select("Select a service to configure:");
    for service in services.iter() {
        let service_type_id = service_versions
            .iter()
            .find(|x| x.id == service.service_version_id)
            .ok_or(eyre!(
                "Failed to map environment service to service version."
            ))?
            .service_type_id;
        let service_type = service_types
            .iter()
            .find(|x| x.id == service_type_id)
            .ok_or(eyre!("Failed to map environment service to service type."))?;
        select_service = select_service.item(service, &service.name, &service_type.name);
    }
    let selected_service = select_service.interact()?;

    let files = ctx.db.list_environment_service_files(selected_service.id)?;

    let mut select_file = cliclack::select("Which file?");
    for file in files.iter() {
        let service_type_file = service_type_files
            .iter()
            .find(|x| x.id == file.service_type_file_id)
            .ok_or(eyre!(
                "Failed to map environment service file to service type file."
            ))?;

        select_file = select_file.item(
            file.id,
            &service_type_file.filename,
            &service_type_file.destination_dir,
        );
    }
    let selected_file = select_file.interact()?;

    // let editor = scrawl::with(&"Hello, world!");
    // let output = editor
    //     .expect("failed to open default editor");
    // let text = output.to_string()
    //     .expect("failed to convert output to string");
    // cliclack::log::remark(text)?;
    Ok(())
}
