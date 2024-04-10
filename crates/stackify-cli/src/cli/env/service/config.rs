use clap::Args;
use cliclack::intro;
use color_eyre::{eyre::eyre, Result};
use stackify_common::{types::EnvironmentName, ConfigElementKind};

use crate::{
    cli::{context::CliContext, theme::ThemedObject},
    util::FindById,
};

#[derive(Debug, Args)]
pub struct ServiceConfigArgs {
    /// The name of the environment to which the service belongs. You can omit
    /// this argument if the service is unique across all environments, otherwise
    /// you will receive an error.
    #[arg(required = false, value_name = "ENVIRONMENT")]
    pub env_name: String,
}

pub fn exec(ctx: &CliContext, args: ServiceConfigArgs) -> Result<()> {
    intro("Configure Environment Services".bold())?;
    let env_name = EnvironmentName::new(&args.env_name)?;
    let env = ctx.db.get_environment_by_name(env_name.as_ref())?;
    let service_types = ctx.db.list_service_types()?;
    let service_versions = ctx.db.list_service_versions()?;
    let service_type_files = ctx.db.list_service_type_files()?;
    let services = ctx
        .db
        .list_environment_services_for_environment_id(env.id)?;

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

    // Retrieve information about the
    let selected_service = select_service.interact()?;
    let selected_service_version = service_versions
        .find_by_id(selected_service.service_version_id)
        .ok_or(eyre!(
            "Failed to map environment service to service version."
        ))?;
    let selected_service_type = service_types
        .find_by_id(selected_service_version.service_type_id)
        .ok_or(eyre!("Failed to map environment service to service type."))?;

    let params = ctx
        .db
        .list_service_type_params_for_service_type(selected_service_type.id)?;

    let selected_param = cliclack::select("Select a parameter to modify:")
        .items(
            &params
                .iter()
                .map(|x| (x.clone(), x.name.clone(), x.description.clone()))
                .collect::<Vec<_>>(),
        )
        .interact()?;

    let files = ctx.db.list_environment_service_files(selected_service.id)?;

    let mut select_config_element = cliclack::select("What would you like to modify?");
    let mut config_elements = Vec::new();
    for file in files.iter() {
        let service_type_file = service_type_files
            .iter()
            .find(|x| x.id == file.service_type_file_id)
            .ok_or(eyre!(
                "Failed to map environment service file to service type file."
            ))?;

        config_elements.push(ConfigElement {
            kind: ConfigElementKind::File,
            id: file.id,
            name: service_type_file.filename.clone(),
            help_text: service_type_file.description.clone(),
        });
    }

    for param in ctx
        .db
        .list_service_type_params_for_service_type(selected_service_type.id)?
    {
        config_elements.push(ConfigElement {
            kind: ConfigElementKind::Param,
            id: param.id,
            name: param.name.clone(),
            help_text: param.description.clone(),
        });
    }

    select_config_element = select_config_element.items(
        &config_elements
            .into_iter()
            .map(|x| (x.clone(), x.name, x.help_text))
            .collect::<Vec<_>>(),
    );

    let selected_element = select_config_element.interact()?;

    // let editor = scrawl::with(&"Hello, world!");
    // let output = editor
    //     .expect("failed to open default editor");
    // let text = output.to_string()
    //     .expect("failed to convert output to string");
    // cliclack::log::remark(text)?;
    Ok(())
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct ConfigElement {
    kind: ConfigElementKind,
    id: i32,
    name: String,
    help_text: String,
}
