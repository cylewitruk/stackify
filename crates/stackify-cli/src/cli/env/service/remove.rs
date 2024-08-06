use color_eyre::{eyre::eyre, Result};
use stackify_common::types::EnvironmentName;

use crate::cli::{context::CliContext, theme::ThemedObject};

use super::ServiceRemoveArgs;

pub fn exec(ctx: &CliContext, args: ServiceRemoveArgs) -> Result<()> {
    let env_name = EnvironmentName::new(&args.env_name)?;
    let env = ctx.db.get_environment_by_name(env_name.as_ref())?;

    let services = ctx
        .db
        .list_environment_services_for_environment_id(env.id)?;
    let service_types = ctx.db.list_service_types()?;
    let service_versions = ctx.db.list_service_versions()?;

    let mut rm_service = cliclack::select("Select a service to remove:");
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
        rm_service = rm_service.item(service, &service.name, &service_type.name);
    }
    let selected_service = rm_service.interact()?;

    let confirm = cliclack::confirm(format!(
        "Are you sure you want to completely remove the service {} from {}?",
        selected_service.name.bold(),
        env.name.bold()
    ))
    .interact()?;

    todo!()
}
