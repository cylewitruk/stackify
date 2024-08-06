use std::collections::HashMap;

use crate::cli::{context::CliContext, theme::ThemedObject};
use crate::cli::{finished, warn};

use clap::{Args, Subcommand};
use color_eyre::Result;
use prettytable::{row, Table};
use stackify_common::types::EnvironmentName;

#[derive(Debug, Args)]
pub struct EpochArgs {
    #[command(subcommand)]
    pub commands: EpochSubCommands,
}

#[derive(Debug, Subcommand)]
pub enum EpochSubCommands {
    /// Prints the current epoch-map for the specified environment.
    List(EpochListArgs),
    /// Modify the epoch-map for the specified environment.
    Edit(EpochEditArgs),
}

#[derive(Debug, Args)]
pub struct EpochListArgs {
    /// The name of the environment to which the epoch-map belongs.
    #[arg(required = true, value_name = "ENVIRONMENT")]
    pub env_name: String,
}

#[derive(Debug, Args)]
pub struct EpochEditArgs {
    /// The name of the environment to which the epoch-map belongs.
    #[arg(required = true, value_name = "ENVIRONMENT")]
    pub env_name: String,
}

pub fn exec_epoch(ctx: &CliContext, args: EpochArgs) -> Result<()> {
    match args.commands {
        EpochSubCommands::List(inner_args) => exec_list(ctx, inner_args),
        EpochSubCommands::Edit(inner_args) => exec_edit(ctx, inner_args),
    }
}

fn exec_list(ctx: &CliContext, args: EpochListArgs) -> Result<()> {
    let env_name = EnvironmentName::new(&args.env_name)?;
    let env = ctx.db.get_environment_by_name(env_name.as_ref())?;

    let all_epochs = ctx.db.list_epochs()?;
    let env_epochs = ctx.db.list_environment_epochs(env.id)?;

    let mut epochs = env_epochs
        .into_iter()
        .map(|e| {
            let epoch = all_epochs
                .iter()
                .find(|epoch| epoch.id == e.epoch_id)
                .unwrap();
            EpochRow {
                env_epoch_id: epoch.id,
                name: epoch.name.clone(),
                starts_at_block_height: e.starts_at_block_height,
                ends_at_block_height: e.ends_at_block_height,
            }
        })
        .collect::<Vec<_>>();
    epochs.sort_by_key(|e| e.starts_at_block_height);

    let mut table = Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_BOX_CHARS);
    table.set_titles(row![
        "Name".table_header(),
        "Block".table_header(),
        "Ends At".table_header(),
    ]);

    for epoch in epochs {
        table.add_row(row![
            epoch.name.magenta().bold(),
            epoch.starts_at_block_height.to_string(),
            epoch
                .ends_at_block_height
                .map(|v| v.to_string())
                .unwrap_or_else(|| "N/A".to_string()),
        ]);
    }

    println!("{table}");

    Ok(())
}

fn exec_edit(ctx: &CliContext, args: EpochEditArgs) -> Result<()> {
    todo!();
    // let env_name = EnvironmentName::new(&args.env_name)?;
    // let env = ctx.db.get_environment_by_name(env_name.as_ref())?;

    // let all_epochs = ctx.db.list_epochs()?;
    // let env_epochs = ctx.db.list_environment_epochs(env.id)?;

    // let mut epochs = env_epochs
    //     .into_iter()
    //     .map(|e| {
    //         let epoch = all_epochs
    //             .iter()
    //             .find(|epoch| epoch.id == e.epoch_id)
    //             .unwrap();
    //         EpochRow {
    //             env_epoch_id: e.id,
    //             name: epoch.name.clone(),
    //             starts_at_block_height: e.starts_at_block_height,
    //             ends_at_block_height: e.ends_at_block_height,
    //         }
    //     })
    //     .collect::<Vec<_>>();
    // epochs.sort_by_key(|e| e.env_epoch_id);

    // let mut last_block_height = i32::MIN;

    // let mut updates: HashMap<i32, i32> = HashMap::new();

    // for i in 0..epochs.len() {
    //     let epoch = &epochs[i];
    //     let msg = format!(
    //         "New block height for epoch '{}':",
    //         epoch.name.magenta().bold()
    //     );

    //     let starts_at_str = epoch.starts_at_block_height.to_string();

    //     // Create our prompt.
    //     let mut new_bh_prompt = Text::new(&msg)
    //         .with_default(&starts_at_str)
    //         .with_placeholder(&starts_at_str)
    //         .with_validator(move |input: &str| match input.parse::<u32>() {
    //             Ok(height) => {
    //                 if i != 0 && height <= last_block_height as u32 {
    //                     return Ok(Validation::Invalid("The block height must be greater than the previous epoch's block height.".into()));
    //                 }
    //                 if i == 0 && height != 0 {
    //                     return Ok(Validation::Invalid("The first epoch must start at block height 0.".into()));
    //                 }
    //                 Ok(Validation::Valid)
    //             },
    //             Err(_) => Ok(Validation::Invalid("The block height must be a valid positive integer.".into())),
    //         });

    //     // Adjust the help text based on the current epoch.
    //     let help_msg;
    //     if i > 0 {
    //         help_msg = format!(
    //             "Enter a block height greater than the previous epoch's block height ({last_block_height})"
    //         );
    //         new_bh_prompt = new_bh_prompt.with_help_message(&help_msg);
    //     } else {
    //         help_msg =
    //             "This is the first epoch, it's starting block height must be zero.".to_string();
    //         new_bh_prompt = new_bh_prompt.with_help_message(&help_msg);
    //     }
    //     new_bh_prompt = new_bh_prompt.with_help_message(&help_msg);

    //     // Prompt the user.
    //     let new_bh = new_bh_prompt.prompt()?;

    //     let new_bh = new_bh.parse::<u32>()?;
    //     updates.insert(epoch.env_epoch_id, new_bh as i32);
    //     last_block_height = new_bh as i32;
    // }

    // println!("");
    // let do_update =
    //     Confirm::new("Are you sure you want to update the block heights for these epochs?")
    //         .prompt()?;
    // if do_update {
    //     ctx.db.update_environment_epochs(updates)?;
    //     println!("");
    //     warn("If you have services which are set to perform actions at certain block heights or epochs, note that these changes may affect their order of operations.");
    //     println!("");
    //     finished(&format!(
    //         "Epochs for environment {} have been updated.",
    //         env_name.magenta().bold()
    //     ));
    // }

    // Ok(())
}

#[derive(Debug)]
struct EpochRow {
    env_epoch_id: i32,
    name: String,
    starts_at_block_height: i32,
    ends_at_block_height: Option<i32>,
}
