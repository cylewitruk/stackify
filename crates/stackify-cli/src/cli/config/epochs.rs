use color_eyre::Result;
use console::style;

use crate::{
    cli::{error, info, success},
    context::CliContext,
    util::FilterByMinMaxEpoch,
};

use super::args::{AddEpochArgs, EpochsArgs, EpochsSubCommands};

pub fn exec(ctx: &CliContext, args: EpochsArgs) -> Result<()> {
    match args.subcommands {
        EpochsSubCommands::List => exec_list_epochs(ctx),
        EpochsSubCommands::Add(inner_args) => exec_add_epoch(ctx, inner_args),
        EpochsSubCommands::Remove => exec_remove_epoch(ctx),
        EpochsSubCommands::Inspect => exec_inspect_epoch(ctx),
    }
}

fn exec_add_epoch(ctx: &CliContext, args: AddEpochArgs) -> Result<()> {
    let epochs = ctx.db.list_epochs()?;
    let highest_epoch = epochs.iter().max_by_key(|e| e.name.clone()).unwrap();
    let highest_block_height = highest_epoch.default_block_height;

    if epochs
        .iter()
        .filter(|e| e.name == args.name.to_string())
        .count()
        > 0
    {
        error("An epoch with that name already exists.");
        return Ok(());
    }

    if args.name <= highest_epoch.name.parse()? {
        error(format!(
            "The epoch must be greater than the current highest epoch: {}.",
            style(&highest_epoch.name).magenta()
        ));
        return Ok(());
    }

    let block_height = match args.block_height {
        Some(height) => {
            if height <= 0 {
                error("Block height must be greater than zero.");
                return Ok(());
            }
            if height <= highest_block_height {
                error(format!(
                    "Block height must be greater than the current highest block height: {}.",
                    style(highest_block_height).cyan()
                ));
                return Ok(());
            }
            height
        }
        None => highest_epoch.default_block_height + 3,
    };

    let new_epoch = ctx
        .db
        .new_epoch(&args.name.to_string(), block_height as u32)?;

    success(format!(
        "Added epoch {} with block height {}.",
        style(&new_epoch.name).magenta().bold(),
        style(new_epoch.default_block_height).cyan()
    ));

    Ok(())
}

fn exec_remove_epoch(_ctx: &CliContext) -> Result<()> {
    todo!()
}

fn exec_inspect_epoch(_ctx: &CliContext) -> Result<()> {
    todo!()
}

fn exec_list_epochs(ctx: &CliContext) -> Result<()> {
    let epochs = ctx.db.list_epochs()?;
    let service_versions = ctx.db.list_service_versions()?;
    let service_upgrade_paths = ctx.db.list_service_upgrade_paths()?;

    println!("{}", style("Supported Epochs:").bold().white());

    for epoch in epochs.iter() {
        let usages_min = service_versions.filter_by_max_epoch(epoch.id);
        let usages_max = service_versions.filter_by_min_epoch(epoch.id);
        let service_version_usages = [usages_min, usages_max].concat();

        let upgrades_min = service_upgrade_paths.filter_by_min_epoch(epoch.id);
        let upgrades_max = service_upgrade_paths.filter_by_max_epoch(epoch.id);
        let upgrade_usages = [upgrades_min, upgrades_max].concat();

        println!("‣ Epoch {}", style(&epoch.name).magenta().bold());
        println!(
            "    {} {}",
            style("block height:").dim(),
            style(epoch.default_block_height).white()
        );

        if !service_version_usages.is_empty() {
            println!(
                "    {} {}",
                style("service versions:").dim(),
                style(service_version_usages.len()).white()
            );
        }

        if !upgrade_usages.is_empty() {
            println!(
                "    {} {}",
                style("upgrade paths:").dim(),
                style(upgrade_usages.len()).white()
            );
        }
    }

    println!("");
    info("Block heights displayed here are the default block heights for each epoch");
    println!("     and can be overridden in each environment.");

    Ok(())
}
