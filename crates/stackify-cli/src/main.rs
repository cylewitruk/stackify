use clap::{CommandFactory, Parser};
use cli::{context::CliContext, StackifyHostDirs};

use color_eyre::eyre::{eyre, Result};
use db::apply_db_migrations;
use diesel::{Connection, SqliteConnection};
use docker::api::DockerApi;
use tokio::sync::broadcast;

use crate::{
    cli::{log::get_log, Cli, Commands},
    errors::ReportResultExt,
};

mod cli;
mod db;
mod docker;
mod errors;
mod includes;
mod util;

#[tokio::main]
async fn main() -> Result<()> {
    let context = initialize().await?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Install(args) => {
            cli::install::exec(&context, args).await.handle()?;
        }
        Commands::Uninstall => {
            cli::uninstall::exec(&context).await.handle()?;
        }
        Commands::Environment(args) => {
            cli::env::exec(&context, args).await.handle()?;
        }
        Commands::Info(args) => {
            cli::info::exec(&context, args).await?;
        }
        Commands::Clean(args) => {
            cli::clean::exec(&context, args).await?;
        }
        Commands::Config(args) => {
            cli::config::exec(&context, args).handle()?;
        }
        Commands::Completions { shell } => {
            shell.generate(&mut Cli::command(), &mut std::io::stdout());
        }
        Commands::MarkdownHelp => {
            clap_markdown::print_help_markdown::<Cli>();
        }
    }

    println!("");
    if cli.dump_logs {
        println!("{}", get_log().join("\n"));
    }
    Ok(())
}

async fn initialize() -> Result<CliContext> {
    env_logger::init();
    color_eyre::install().unwrap();

    let home_dir = home::home_dir().ok_or_else(|| eyre!("Failed to get home directory."))?;

    let app_root = home_dir.join(".stackify");
    std::fs::create_dir_all(&app_root)?;
    let config_file = app_root.join("config.toml");
    let db_file = app_root.join("stackify.db");
    let tmp_dir = app_root.join("tmp");
    if tmp_dir.exists() {
        std::fs::remove_dir_all(&tmp_dir)?;
    }
    std::fs::create_dir(&tmp_dir)?;
    let data_dir = app_root.join("data");
    std::fs::create_dir_all(&data_dir)?;
    let bin_dir = app_root.join("bin");
    std::fs::create_dir_all(&bin_dir)?;
    let assets_dir = app_root.join("assets");
    std::fs::create_dir_all(&assets_dir)?;

    let mut connection =
        SqliteConnection::establish(&db_file.to_string_lossy()).map_err(|e| eyre!(e))?;

    apply_db_migrations(&mut connection)?;

    let host_dirs = StackifyHostDirs {
        app_root: app_root.clone(),
        bin_dir: bin_dir.clone(),
        tmp_dir: tmp_dir.clone(),
        assets_dir: assets_dir.clone(),
    };

    let docker_api = DockerApi::new(
        host_dirs.clone(),
        docker::StackifyContainerDirs {
            home_dir: std::path::PathBuf::from("/home/stackify/"),
            bin_dir: std::path::PathBuf::from("/opt/stackify/bin/"),
            data_dir: std::path::PathBuf::from("/opt/stackify/data/"),
            config_dir: std::path::PathBuf::from("/opt/stackify/config/"),
            logs_dir: std::path::PathBuf::from("/var/log/stackify/"),
        },
    )
    .await?;

    let (tx, _) = broadcast::channel::<()>(10);

    let context = CliContext::new(host_dirs, docker_api, tx.clone()).await?;

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        let _ = tx.send(());
    });

    Ok(context)
}
