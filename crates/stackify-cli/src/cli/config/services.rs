use crate::db::model::{Epoch, ServiceType, ServiceUpgradePath, ServiceVersion};
use color_eyre::eyre::{eyre, Result};
use console::style;
use stackify_common::util::to_alphanumeric_snake;

use crate::{
    context::CliContext,
    util::{
        git::{GitTarget, TargetType},
        FilterByServiceType, FilterByServiceVersion, FindByCliName, FindById,
    },
};

use super::args::{
    AddServiceVersionArgs, RemoveServiceVersionArgs, ServiceSubCommands, ServicesArgs,
};

pub fn exec_services(ctx: &CliContext, args: ServicesArgs) -> Result<()> {
    match args.subcommands {
        ServiceSubCommands::List => exec_list_services(ctx),
        ServiceSubCommands::AddVersion(inner_args) => exec_add_service_version(ctx, inner_args),
        ServiceSubCommands::RemoveVersion(inner_args) => {
            exec_remove_service_version(ctx, inner_args)
        }
        ServiceSubCommands::Inspect => exec_inspect_service(ctx),
    }
}

pub fn exec_add_service_version(ctx: &CliContext, args: AddServiceVersionArgs) -> Result<()> {
    let cli_name = to_alphanumeric_snake(&args.svc_name);

    let svc_versions = ctx.db.list_service_versions()?;
    if svc_versions.find_by_cli_name(&cli_name).is_some() {
        return Err(eyre!(
            "A service version with the name '{}' already exists",
            cli_name
        ));
    }

    Ok(())
}

pub fn exec_remove_service_version(
    _ctx: &CliContext,
    args: RemoveServiceVersionArgs,
) -> Result<()> {
    let _cli_name = to_alphanumeric_snake(&args.svc_name);

    Ok(())
}

pub fn exec_inspect_service(_ctx: &CliContext) -> Result<()> {
    Ok(())
}

pub fn exec_list_services(ctx: &CliContext) -> Result<()> {
    // Fetch service data from the database.
    let epochs = ctx.db.list_epochs()?;
    let service_types = ctx.db.list_service_types()?;
    let service_versions = ctx.db.list_service_versions()?;
    let service_upgrade_paths = ctx.db.list_service_upgrade_paths()?;

    // Header
    println!("{}", style("Supported Services:").bold().white());

    // Iterate over available service types and print their details.
    for service_type in service_types.iter() {
        // Print the service type name header.
        println!("‣ {}", style(&service_type.name).magenta());

        println!(
            "  {} {} {}",
            style("ƒ").white(),
            to_alphanumeric_snake(&service_type.name),
            style("(cli name)").dim()
        );

        // Get the available versions for this service type.
        let versions = service_versions.filter_by_service_type(service_type.id);

        // Iterate over the available versions and print their details.
        for i in 0..versions.len() {
            let version = versions[i];

            // Print the version header.
            println!(
                "  {} {} {}",
                style("◆").yellow(),
                style(&version.version).cyan(),
                style("(version)").dim()
            );

            println!(
                "  {} {} {} {}",
                style("┆").dim(),
                //style("␂").white(),
                style("ƒ").white(),
                &version.cli_name,
                style("(cli name)").dim()
            );

            // If there is a git target, print it.
            if let Some(target) = GitTarget::parse_opt(version.git_target.clone()) {
                print_git_target(&target);
            }

            // If there is a minimum epoch, print it.
            if let Some(epoch) = epochs.find_by_id_opt(version.minimum_epoch_id) {
                print_minimum_epoch(epoch);
            }

            // If there is a maximum epoch, print it.
            if let Some(epoch) = epochs.find_by_id_opt(version.maximum_epoch_id) {
                print_maximum_epoch(epoch);
            }

            // Print the available upgrade paths.
            print_upgrade_paths(
                &service_types,
                &service_versions,
                &service_upgrade_paths,
                version.id,
            )?;
        }
        //println!("");
    }
    Ok(())
}

fn print_minimum_epoch(epoch: &Epoch) {
    println!(
        "  {} {} {} {}",
        style("┆").dim(),
        style("▼").green(),
        style(&epoch.name),
        style("(minimum epoch)").dim()
    );
}

fn print_maximum_epoch(epoch: &Epoch) {
    println!(
        "  {} {} {} {}",
        style("┆").dim(),
        style("▲").red(),
        style(&epoch.name),
        style("(maximum epoch)").dim()
    );
}

fn print_git_target(target: &GitTarget) {
    let git_type = match target.target_type {
        TargetType::Tag => format!("{}", style("(git tag)").dim()),
        TargetType::Branch => format!("{}", style("(git branch)").dim()),
        TargetType::Commit => format!("{}", style("(git commit)").dim()),
    };

    println!(
        "  {} {} {} {}",
        style("┆").dim(),
        style("☉").bright().blue(),
        target.target,
        style(git_type).dim()
    );
}

fn print_upgrade_paths(
    service_types: &[ServiceType],
    service_versions: &[ServiceVersion],
    service_upgrade_paths: &[ServiceUpgradePath],
    version_id: i32,
) -> Result<()> {
    // Print the available upgrade paths.
    for path in service_upgrade_paths.filter_by_service_version(version_id) {
        let to_service_version = service_versions
            .find_by_id(path.to_service_version_id)
            .ok_or(eyre!("Failed to find service version"))?;
        let to_service_type = service_types
            .find_by_id(to_service_version.service_type_id)
            .ok_or(eyre!("Failed to find service type"))?;
        println!(
            "  {} {} {}: {} {}",
            style("┆").dim(),
            style("⤑").green(),
            style(&to_service_type.name),
            style(&to_service_version.version).green(),
            style("(upgradable to)").dim()
        );
    }

    Ok(())
}
