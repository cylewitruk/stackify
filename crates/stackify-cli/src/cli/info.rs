use std::fmt::Debug;

use clap::Args;
use color_eyre::{eyre::{eyre, Result}, owo_colors::OwoColorize};
use comfy_table::{Cell, CellAlignment, ColumnConstraint, Table, Width};
use console::style;
use regex::Regex;

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
    services: bool
}

pub fn exec(ctx: &CliContext, args: InfoArgs) -> Result<()> {
    println!("{}", 
        format!("Stackify CLI v{}", env!("CARGO_PKG_VERSION"))
            .fg_rgb::<255, 165, 0>());
    println!("");
    println!("{}", style("Stackify Status:").bold().white());
    println!("  ‣ Environments: {}", ctx.db.list_environments()?.len());
    println!("  ‣ Docker Images: {}", ctx.docker.list_stackify_images()?.len());

    if args.docker {
        println!("");
        exec_display_docker_info(ctx)?;
    }

    if args.epochs {
        println!("");
        exec_list_epochs(ctx)?;
    }
    
    if args.services {
        println!("");
        exec_list_service_types(ctx)?;
    }

    Ok(())
}

fn exec_display_docker_info(ctx: &CliContext) -> Result<()> {
    let docker_version = ctx.docker.get_docker_version()?;

    println!("{}", style("Docker Information").bold().white());
    println!("  ‣ Version: {}", docker_version.version);
    println!("  ‣ API Version: {}", docker_version.api_version);
    println!("  ‣ Min API Version: {}", docker_version.min_api_version);
    println!("");

    println!("{}", "Stackify Images".bold());
    let images = ctx.docker.list_stackify_images()?;
    if images.is_empty() {
        println!(" ‣ No images found");
    } else {
        let mut table = Table::new();
        table
            .set_header(vec![
                "IMAGE".cyan(),
                "TAG".cyan(),
                "CONTAINERS".cyan(),
                "SIZE".cyan(),
            ])
            .load_preset(comfy_table::presets::NOTHING);

        table.column_mut(0)
            .ok_or(eyre!("Failed to retrieve column."))?
            .set_constraint(ColumnConstraint::LowerBoundary(Width::Fixed(25)));

            table.column_mut(1)
            .ok_or(eyre!("Failed to retrieve column."))?
            .set_constraint(ColumnConstraint::LowerBoundary(Width::Fixed(15)));

        table.column_mut(2)
            .ok_or(eyre!("Failed to retrieve column."))?
            .set_cell_alignment(CellAlignment::Right);

        table.column_mut(3)
            .ok_or(eyre!("Failed to retrieve column."))?
            .set_cell_alignment(CellAlignment::Right);

        let regex = Regex::new(r#"^([^:]+)(:(.+))?$"#)?;
        for image in images {
            for tag in image.tags {
                let captures = regex.captures(&tag).ok_or(eyre!("Failed to capture regex"))?;
                let repository = captures.get(1).ok_or(eyre!("Failed to get repository"))?.as_str();
                let tag = captures.get(3).ok_or(eyre!("Failed to get tag"))?.as_str();
                table.add_row(vec![
                    Cell::new(&repository),
                    Cell::new(&tag),
                    Cell::new(if image.container_count == -1 { "0".to_string() } else { image.container_count.to_string() }),
                    Cell::new((image.size / 1024 / 1024).to_string() + "MB")
                ]);
            }
        }

        println!("{table}");
    }

    Ok(())
}

fn exec_list_service_types(ctx: &CliContext) -> Result<()> {
    let epochs = ctx.db.list_epochs()?;
    let service_types = ctx.db.list_service_types()?;
    let service_versions = ctx.db.list_service_versions()?;
    let service_upgrade_paths = ctx.db.list_service_upgrade_paths()?;

    println!("{}", style("Supported Services:").bold().white());
    for service_type in service_types.iter() {
        println!("  ‣ {}", style(&service_type.name).magenta().bold());
        let versions = service_versions
            .iter()
            .filter(|v| v.service_type_id == service_type.id)
            .collect::<Vec<_>>();
        for i in 0..versions.len() {
            let version = versions[i];
            // println!("    ᛭ {} {}", 
            //     style("Version").dim(), 
            //     style(&version.version).bold()
            // );
            println!("    {} {}", style("Version").white(), style(&version.version).cyan().bold());

            if version.git_target.is_some() {
                let git_target = version.git_target.as_ref().unwrap();
                let split = git_target.split(":").collect::<Vec<_>>();
                let git_type = match split[0] {
                    "tag" => format!("{}", style("Git tag:").dim()),
                    "branch" => format!("{}", style("Git branch:").dim()),
                    "commit" => format!("{}", style("Git commit:").dim()),
                    _ => "Unknown".into()
                };
                
                println!("      {} {} {}", 
                    style("☉").blue(), 
                    style(git_type).dim(), 
                    split[1].bold());
            }

            if version.minimum_epoch_id.is_some() {
                let epoch = epochs
                    .iter()
                    .find(|e| e.id == version.minimum_epoch_id.unwrap())
                    .ok_or(eyre!("Failed to find epoch"))?;
                println!("      {} {} {}", style("▼").green(), style("Minimum epoch:").dim(), style(&epoch.name).bold());
            }

            if version.maximum_epoch_id.is_some() {
                let epoch = epochs
                    .iter()
                    .find(|e| e.id == version.maximum_epoch_id.unwrap())
                    .ok_or(eyre!("Failed to find epoch"))?;
                println!("      {} {} {}", style("▲").red(), style("Maximum epoch:").dim(), style(&epoch.name).bold());
            }
            
            let upgrade_paths = service_upgrade_paths
                .iter()
                .filter(|p| p.from_service_version_id == version.id)
                .collect::<Vec<_>>();
            for path in upgrade_paths {
                let to_service_version = service_versions
                    .iter()
                    .find(|v| v.id == path.to_service_version_id)
                    .ok_or(eyre!("Failed to find service version"))?;
                let to_service_type = service_types
                    .iter()
                    .find(|t| t.id == to_service_version.service_type_id)
                    .ok_or(eyre!("Failed to find service type"))?;
                println!("      {} {} {} ({})", 
                    style("⤑").green(), 
                    style("Upgradable to:").dim(), 
                    style(&to_service_type.name).bold(), 
                    style(&to_service_version.version).green()
                );
            }

            if i < versions.len() - 1 {
                println!("");
            }
        }
        println!("");
    }
    Ok(())
}

fn exec_list_epochs(_ctx: &CliContext) -> Result<()> {
    println!("List epochs");
    Ok(())
}