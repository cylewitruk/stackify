use crate::db::model::{Epoch, ServiceType, ServiceUpgradePath, ServiceVersion};
use color_eyre::eyre::{bail, eyre, Result};
use console::style;
use inquire::{Select, Text};
use stackify_common::util::to_alphanumeric_snake;

use crate::{
    cli::context::CliContext,
    util::{
        git::{GitTarget, TargetType},
        FilterByServiceType, FilterByServiceVersion, FindById,
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
    let service_types = ctx.db.list_service_types()?;

    // Collect service type
    let service_type_names = service_types.iter().map(|st| st.name.clone()).collect::<Vec<_>>();
    let service_type = Select::new("Select a service type", service_type_names)
        .prompt()?;
    let service_type = service_types.iter().find(|st| st.name == service_type)
        .ok_or(eyre!("Service type not found"))?;
    
    // Collect service version
    let all_service_versions = ctx.db.list_service_versions()?;
    let service_versions = all_service_versions
        .filter_by_service_type(service_type.id);
    let service_version_names = service_versions.iter().map(|sv| sv.version.clone()).collect::<Vec<_>>();
    let service_version = Select::new("Select a service version", service_version_names)
        .prompt()?;
    let service_version_id = service_versions.iter().find(|sv| sv.version == service_version)
        .ok_or(eyre!("Service version not found"))?;

    if service_type.allow_git_target {
        let git_target_type = Select::new("What kind of git target do you want to use?", vec!["Tag", "Branch", "Commit hash"])
            .prompt()?;
        let git_target = match git_target_type {
            "Tag" => {
                Text::new("Enter the git target")
                    .with_help_message("This is the tag of the 'stacks-core' Github repository from which the service should be built.")
                    .prompt()?
            }
            "Branch" => {
                Text::new("Enter the git target")
                    .with_help_message("This is the branch of the 'stacks-core' Github repository from which the service should be built.")
                    .prompt()?
            }
            "Commit hash" => {
                Text::new("Enter the git target")
                    .with_help_message("This is the commit hash of the 'stacks-core' Github repository from which the service should be built.")
                    .prompt()?
            }
            _ => bail!("Invalid git target type"),
        };
    }

    todo!()
}

pub fn exec_remove_service_version(
    _ctx: &CliContext,
    args: RemoveServiceVersionArgs,
) -> Result<()> {
    let _cli_name = to_alphanumeric_snake(&args.svc_name);

    todo!()
}

pub fn exec_inspect_service(_ctx: &CliContext) -> Result<()> {
    todo!()
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
