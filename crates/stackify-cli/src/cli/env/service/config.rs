use clap::Args;
use cliclack::intro;
use color_eyre::{
    eyre::{bail, eyre},
    Result,
};
use stackify_common::{types::EnvironmentName, ConfigElementKind, ServiceType, ValueType};

use crate::{
    cli::{context::CliContext, theme::ThemedObject},
    db::cli_db::CliDatabase,
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
    let env = ctx.db.load_environment(env_name.as_ref())?;
    let service_types = ctx.db.list_service_types()?;
    let service_versions = ctx.db.list_service_versions()?;
    // let service_type_files = ctx.db.list_service_type_files()?;
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

    match ValueType::from_i32(selected_param.value_type_id)? {
        ValueType::Boolean => {
            let value = cliclack::select("Select a value:")
                .items(&[(true, "true", ""), (false, "false", "")])
                .interact()?;
            ctx.db.set_service_param_value(
                selected_service.id,
                selected_param.id,
                value.to_string(),
            )?;
        }
        ValueType::Integer => {
            let value: String = cliclack::input("Enter a value:").interact()?;
            let value = value.parse::<i32>()?;
            ctx.db.set_service_param_value(
                selected_service.id,
                selected_param.id,
                value.to_string(),
            )?;
        }
        ValueType::String => {
            let value = cliclack::input("Enter a value:").interact()?;
            ctx.db
                .set_service_param_value(selected_service.id, selected_param.id, value)?;
        }
        // ValueType::Enum => {
        //     let allowed_values = ctx
        //         .db
        //         .list_service_type_param_allowed_values(selected_param.id)?;
        //     let value = cliclack::select("Select a value:")
        //         .items(
        //             &allowed_values
        //                 .iter()
        //                 .map(|x| (x.clone(), x.value.clone(), x.description.clone()))
        //                 .collect::<Vec<_>>(),
        //         )
        //         .interact()?;
        //     ctx.db.set_service_param_value(selected_param.id, value.value)?;
        // },
        ValueType::StacksKeychain => {
            let value = cliclack::input("Enter a value:").interact()?;
            ctx.db
                .set_service_param_value(selected_service.id, selected_param.id, value)?;
        }
        ValueType::Service => match ServiceType::from_i32(selected_service_type.id)? {
            ServiceType::StacksSigner => {
                let stacks_peers = env
                    .services
                    .iter()
                    .filter(|service| {
                        [ServiceType::StacksMiner, ServiceType::StacksFollower]
                            .contains(&ServiceType::from_i32(service.service_type.id).unwrap())
                    })
                    .filter(|svc| &svc.name != &selected_service.name)
                    .map(|service| service.name.clone())
                    .collect::<Vec<_>>();

                let stacks_node =
                    cliclack::select("Which Stacks node should this signer receive events from?")
                        .items(
                            &stacks_peers
                                .iter()
                                .map(|sn| (sn.clone(), sn, ""))
                                .collect::<Vec<_>>(),
                        )
                        .interact()?;

                ctx.db.set_service_param_value(
                    selected_service.id,
                    selected_param.id,
                    stacks_node,
                )?;
            }
            _ => bail!("Unsupported service type for ValueType::Service"),
        },
        _ => bail!("Unsupported value type"),
    }

    cliclack::outro_note(
        "Configuration Updated".green().bold(),
        format!(
            "The configuration parameter {} has been updated for {}.",
            selected_param.name.cyan(),
            env_name.magenta()
        ),
    )?;

    Ok(())
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct ConfigElement {
    kind: ConfigElementKind,
    id: i32,
    name: String,
    help_text: String,
}
