use clap::{CommandFactory, Parser};
use color_eyre::eyre::{eyre, Result};
use cli::context::CliContext;
use db::{apply_db_migrations, AppDb};
use diesel::{Connection, SqliteConnection};
use stackify_common::docker::stackify_docker::StackifyDocker;

use crate::cli::{Cli, Commands};

mod cli;
mod db;
mod util;

fn main() -> Result<()> {
    let context = initialize()?;

    match Cli::try_parse() {
        Ok(cli) => match cli.command {
            Commands::Initialize(args) => {
                cli::init::exec(&context, args)?;
            }
            Commands::Environment(args) => {
                cli::env::exec(&context, args)?;
            }
            Commands::Info(args) => {
                cli::info::exec(&context, args)?;
            }
            Commands::Clean(args) => {
                println!("Clean");
                cli::clean::exec(&context, args)?;
            }
            Commands::Config(args) => {
                cli::config::exec(&context, args)?;
            }
            Commands::Completions { shell } => {
                shell.generate(&mut Cli::command(), &mut std::io::stdout());
            },
            Commands::MarkdownHelp => {
                clap_markdown::print_help_markdown::<Cli>();
            }
        },
        Err(e) => {
            e.print()?;
        }
    }

    println!("");
    Ok(())
}

fn initialize() -> Result<CliContext> {
    env_logger::init();
    color_eyre::install().unwrap();

    let uid;
    let gid;
    unsafe {
        uid = libc::geteuid();
        gid = libc::getegid();
    }

    let home_dir = home::home_dir().ok_or_else(|| eyre!("Failed to get home directory."))?;

    let config_dir = home_dir.join(".stackify");
    std::fs::create_dir_all(&config_dir)?;
    let config_file = config_dir.join("config.toml");
    let db_file = config_dir.join("stackify.db");
    let tmp_dir = config_dir.join("tmp");
    if tmp_dir.exists() {
        std::fs::remove_dir_all(&tmp_dir)?;
    }
    std::fs::create_dir(&tmp_dir)?;
    let data_dir = config_dir.join("data");
    std::fs::create_dir_all(&data_dir)?;
    let bin_dir = config_dir.join("bin");
    std::fs::create_dir_all(&bin_dir)?;

    let mut connection =
        SqliteConnection::establish(&db_file.to_string_lossy()).map_err(|e| eyre!(e))?;

    apply_db_migrations(&mut connection)?;

    let app_db = AppDb::new(connection);

    let docker = StackifyDocker::new()?;

    let context = CliContext {
        config_dir,
        config_file,
        data_dir,
        bin_dir,
        tmp_dir,
        db_file,
        db: app_db,
        user_id: uid,
        group_id: gid,
        docker,
    };

    Ok(context)
}
