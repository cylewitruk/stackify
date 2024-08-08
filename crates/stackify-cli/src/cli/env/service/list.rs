use color_eyre::{eyre::eyre, Result};
use prettytable::row;
use stackify_common::types::EnvironmentName;

use crate::{
    cli::{context::CliContext, theme::ThemedObject},
    db::cli_db::CliDatabase as _,
    util::{print, FindById as _},
};

use super::ServiceListArgs;

pub fn exec(ctx: &CliContext, args: ServiceListArgs) -> Result<()> {
    let env_name = EnvironmentName::new(&args.env_name)?;

    cliclack::intro(format!(
        "Listing services for environment '{}'",
        &env_name.bold().magenta()
    ))?;

    let db = ctx.clidb();
    let env = db.load_environment(&env_name)?;

    let mut table = prettytable::Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_BOX_CHARS);
    table.set_titles(row![
        "Service (Name)".cyan().bold(),
        "Type".cyan().bold(),
        "Version".cyan().bold(),
        "Remark".cyan().bold()
    ]);

    for svc in env.services.iter() {
        table.add_row(row![
            svc.name,
            svc.service_type.name,
            svc.version.version,
            svc.remark.clone().unwrap_or("<none>".to_string()).gray()
        ]);
    }

    let mut lines = vec![];
    table.print(&mut lines)?;

    let table_str = String::from_utf8_lossy(&lines);
    for line in table_str.lines() {
        println!("{} {}", "│".bright_black(), line);
    }
    cliclack::outro(format!("{} services", env.services.len()))?;

    Ok(())
}
