use color_eyre::{eyre::eyre, Result};
use prettytable::row;
use stackify_common::types::EnvironmentName;

use crate::{
    cli::{context::CliContext, env::args::ServiceListArgs, theme::ThemedObject},
    db::cli_db::CliDatabase as _,
    util::FindById as _,
};

pub fn exec(ctx: &CliContext, args: ServiceListArgs) -> Result<()> {
    let env = if let Some(env_name) = args.env_name {
        EnvironmentName::new(&env_name)?;
        Some(ctx.db.get_environment_by_name(env_name.as_str())?)
    } else {
        None
    };

    if env.is_some() {
        list_for_environment_id(ctx, env.unwrap().id)
    } else {
        list_for_all_environments_2(ctx)
    }
}

fn list_for_all_environments_2(ctx: &CliContext) -> Result<()> {
    println!("{}", "Listing services for all environments:");
    let db = ctx.clidb();

    let environments = db.load_all_environments()?;
    for env in environments {
        println!("");

        let mut main_table = prettytable::Table::new();
        main_table.set_format(*prettytable::format::consts::FORMAT_NO_BORDER);
        main_table.add_row(row![env.name.bold()]);

        let mut svc_table = prettytable::Table::new();
        svc_table.set_format(*prettytable::format::consts::FORMAT_BOX_CHARS);
        svc_table.set_titles(row![
            "Service (Name)".cyan().bold(),
            "Type".cyan().bold(),
            "Version".cyan().bold(),
            "Remark".cyan().bold()
        ]);

        for svc in env.services.iter() {
            svc_table.add_row(row![
                svc.name,
                svc.service_type.name,
                svc.version.version,
                svc.remark.clone().unwrap_or("<none>".to_string()).gray()
            ]);
        }

        main_table.add_row(row![svc_table]);
        main_table.printstd();
    }

    Ok(())
}

fn list_for_all_environments(ctx: &CliContext) -> Result<()> {
    let db = ctx.clidb();
    let services = ctx.db.list_environment_services()?;
    let mut environments = ctx.db.list_environments()?;
    let service_versions = ctx.db.list_service_versions()?;
    let service_types = ctx.db.list_service_types()?;

    environments.sort_unstable_by_key(|x| x.name.clone());

    let mut main_table = prettytable::Table::new();
    main_table.set_format(*prettytable::format::consts::FORMAT_NO_BORDER);

    for environment in environments.iter() {
        let env_name = environment.name.bold();
        main_table.add_row(row![env_name.bold()]);

        let mut svc_table = prettytable::Table::new();
        svc_table.set_format(*prettytable::format::consts::FORMAT_BOX_CHARS);
        svc_table.set_titles(row![
            "Service (Name)".cyan().bold(),
            "Type".cyan().bold(),
            "Version".cyan().bold(),
            "Remark".cyan().bold()
        ]);
        for service in services.iter() {
            if service.environment_id == environment.id {
                let service_version = service_versions
                    .find_by_id(service.service_version_id)
                    .ok_or(eyre!(
                        "Failed to map environment service to service version."
                    ))?;

                let service_type = service_types
                    .find_by_id(service_version.service_type_id)
                    .ok_or(eyre!("Failed to map environment service to service type."))?;

                svc_table.add_row(row![
                    service.name,
                    service_type.name.clone(),
                    service_version.version.clone(),
                    service
                        .comment
                        .clone()
                        .unwrap_or("<none>".to_string())
                        .gray()
                ]);
            }
        }

        main_table.add_row(row![svc_table]);
    }

    main_table.printstd();
    Ok(())
}

fn list_for_environment_id(ctx: &CliContext, env_id: i32) -> Result<()> {
    let services = ctx
        .db
        .list_environment_services_for_environment_id(env_id)?;
    todo!()
}
