use color_eyre::Result;

use cliclack::outro_note;
use console::style;
use prettytable::{format::Alignment, row, Cell, Row, Table};

use crate::{
    cli::{context::CliContext, theme::ThemedObject},
    db::cli_db::CliDatabase,
};

use super::args::ListArgs;

pub async fn exec(ctx: &CliContext, _args: ListArgs) -> Result<()> {
    let environments = ctx.db.load_all_environments()?;

    if environments.is_empty() {
        outro_note(
            "No environments found.",
            format!(
                "To create a new environment, use the {} command.",
                style("stackify env create").white().bold()
            ),
        )?;
        return Ok(());
    }

    let mut table = Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_BOX_CHARS);
    table.set_titles(row![
        format!("{:<20}", "Environment".table_header()),
        "Epochs".table_header(),
        "Services".table_header(),
    ]);

    for env in environments {
        // Create the epoch sub-table
        let mut epoch_table = Table::new();
        epoch_table.set_format(*prettytable::format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
        epoch_table.set_titles(Row::new(vec![
            Cell::new(&"Epoch".cyan().to_string()),
            Cell::new_align(&"@Block".cyan().to_string(), Alignment::RIGHT),
        ]));
        for epoch in env.epochs {
            epoch_table.add_row(Row::new(vec![
                Cell::new(&epoch.epoch.name),
                Cell::new_align(&epoch.starts_at_block_height.to_string(), Alignment::RIGHT),
            ]));
        }

        // Create the services sub-table
        let mut service_table = Table::new();
        service_table.set_format(*prettytable::format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
        service_table.set_titles(Row::new(vec![
            Cell::new(&"Service".cyan().to_string()),
            Cell::new(&"Version".cyan().to_string()),
        ]));
        for service in env.services {
            service_table.add_row(Row::new(vec![
                Cell::new(&service.name),
                Cell::new(&service.version.version),
            ]));
        }

        // Print the environment row in the main table
        table.add_row(row![env.name.bold(), epoch_table, service_table,]);
    }

    println!("{table}");

    println!(
        "To see more details about an environment, use the {} command.",
        style("stackify env inspect").white().bold()
    );

    println!(
        "To start an environment, use the {} command.",
        style("stackify env start").white().bold()
    );

    Ok(())
}
