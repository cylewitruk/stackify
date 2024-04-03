use std::fmt::Debug;

use clap::Args;
use color_eyre::{
    eyre::{eyre, Result},
    owo_colors::OwoColorize,
};
use comfy_table::{Cell, CellAlignment, ColumnConstraint, Table, Width};
use console::style;
use docker_api::opts::ImageListOpts;
use regex::Regex;

use super::context::CliContext;

#[derive(Debug, Args)]
pub struct InfoArgs {
    #[arg(short = 'd', long, default_value = "false", required = false)]
    docker: bool,

    #[arg(short = 'e', long, default_value = "false", required = false)]
    epochs: bool,

    #[arg(short = 's', long, default_value = "false", required = false)]
    services: bool,
}

pub async fn exec(ctx: &CliContext, args: InfoArgs) -> Result<()> {
    println!(
        "{}",
        format!("Stackify CLI v{}", env!("CARGO_PKG_VERSION")).fg_rgb::<255, 165, 0>()
    );
    println!("");
    println!("{}", style("Stackify Status:").bold().white());
    println!("‣ Environments: {}", ctx.db.list_environments()?.len());
    println!(
        "‣ Docker Images: {}",
        ctx.docker()
            .api()
            .images()
            .list(&ImageListOpts::default())
            .await?
            .len()
    );

    if args.docker {
        println!("");
        exec_display_docker_info(ctx).await?;
    }

    if args.epochs {
        println!("");
        exec_list_epochs(ctx)?;
    }

    Ok(())
}

async fn exec_display_docker_info(ctx: &CliContext) -> Result<()> {
    let docker_version = ctx.docker().api().version().await?;

    println!("{}", style("Docker Information:").bold().white());
    println!(
        "‣ Version: {}",
        docker_version.kernel_version.unwrap_or_default()
    );
    println!(
        "‣ API Version: {}",
        docker_version.api_version.unwrap_or_default()
    );
    println!(
        "‣ Min API Version: {}",
        docker_version.min_api_version.unwrap_or_default()
    );
    println!("");

    println!("{}", "Stackify Images:".bold());
    let images = ctx
        .docker()
        .api()
        .images()
        //TODO: Add filter for stackify images
        .list(&ImageListOpts::default())
        .await?;
    if images.is_empty() {
        println!("‣ No images found");
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

        table
            .column_mut(0)
            .ok_or(eyre!("Failed to retrieve column."))?
            .set_constraint(ColumnConstraint::LowerBoundary(Width::Fixed(25)));

        table
            .column_mut(1)
            .ok_or(eyre!("Failed to retrieve column."))?
            .set_constraint(ColumnConstraint::LowerBoundary(Width::Fixed(15)));

        table
            .column_mut(2)
            .ok_or(eyre!("Failed to retrieve column."))?
            .set_cell_alignment(CellAlignment::Right);

        table
            .column_mut(3)
            .ok_or(eyre!("Failed to retrieve column."))?
            .set_cell_alignment(CellAlignment::Right);

        let regex = Regex::new(r#"^([^:]+)(:(.+))?$"#)?;
        for image in images {
            for tag in image.repo_tags {
                let captures = regex
                    .captures(&tag)
                    .ok_or(eyre!("Failed to capture regex"))?;
                let repository = captures
                    .get(1)
                    .ok_or(eyre!("Failed to get repository"))?
                    .as_str();
                let tag = captures.get(3).ok_or(eyre!("Failed to get tag"))?.as_str();
                table.add_row(vec![
                    Cell::new(&repository),
                    Cell::new(&tag),
                    Cell::new(if image.containers == -1 {
                        "0".to_string()
                    } else {
                        image.containers.to_string()
                    }),
                    Cell::new((image.size / 1024 / 1024).to_string() + "MB"),
                ]);
            }
        }

        println!("{table}");
    }

    Ok(())
}

fn exec_list_epochs(_ctx: &CliContext) -> Result<()> {
    println!("List epochs");
    Ok(())
}
