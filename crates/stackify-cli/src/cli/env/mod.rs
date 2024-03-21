use color_eyre::eyre::{eyre, Result};
use comfy_table::{Cell, Color, ColumnConstraint, Table, Width};

use crate::context::CliContext;

use self::args::EnvArgs;

pub mod args;

pub fn exec(ctx: &CliContext, args: EnvArgs) -> Result<()> {
    match args.commands {
        args::EnvSubCommands::List(inner_args) => exec_list(ctx, inner_args),
        args::EnvSubCommands::Create(inner_args) => exec_create(ctx, inner_args),
        args::EnvSubCommands::Delete(inner_args) => exec_delete(ctx, inner_args),
        args::EnvSubCommands::Start(inner_args) => exec_start(ctx, inner_args),
        args::EnvSubCommands::Stop(inner_args) => exec_stop(ctx, inner_args),
    }
}

fn exec_list(ctx: &CliContext, _args: args::ListArgs) -> Result<()> {
    let environments = ctx.db.list_environments()?;

    let mut table = Table::new();
    table
        .set_header(vec![
            Cell::new("NAME").fg(Color::Cyan),
            Cell::new("CREATED").fg(Color::Cyan),
            Cell::new("LAST RUN").fg(Color::Cyan)
        ])
        .load_preset(comfy_table::presets::NOTHING);

    table.column_mut(0)
        .ok_or(eyre!("Failed to retrieve column."))?
        .set_constraint(ColumnConstraint::LowerBoundary(Width::Fixed(40)));

    for env in environments {
        table.add_row(vec![
            env.name,
            env.created_at.date().to_string(),
            env.updated_at.date().to_string(),
        ]);
    }

    println!("{table}");
    Ok(())
}

fn exec_create(ctx: &CliContext, args: args::CreateArgs) -> Result<()> {
    let env = ctx.db.create_environment(&args.name, args.bitcoin_block_speed)?;
    println!("Environment created: {}", env.id);
    Ok(())
}

fn exec_delete(_ctx: &CliContext, _args: args::DeleteArgs) -> Result<()> {
    println!("Delete environment");
    Ok(())
}

fn exec_start(_ctx: &CliContext, args: args::StartArgs) -> Result<()> {
    println!("Start environment: {}", args.env_name);
    Ok(())
}

fn exec_stop(_ctx: &CliContext, _args: args::StopArgs) -> Result<()> {
    println!("Stop environment");
    Ok(())
}