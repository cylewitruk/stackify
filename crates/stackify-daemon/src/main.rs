use color_eyre::{
    eyre::{bail, eyre},
    Result,
};
use diesel::{connection::SimpleConnection, Connection, SqliteConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use env_logger::Env;
use log::info;

pub mod api;
pub mod control;
pub mod db;
pub mod errors;

pub const DB_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

#[rocket::main]
async fn main() -> Result<()> {
    initialize()?;

    let db_path = "/home/cylwit/.stackify/stackifyd.db";
    let _channel = control::Monitor::new(db_path).start().await?;

    rocket::build()
        .mount("/api", api::routes())
        .launch()
        .await?;

    Ok(())
}

pub fn initialize() -> Result<String> {
    let env = Env::default().filter_or("RUST_LOG", "trace");
    env_logger::init_from_env(env);
    color_eyre::install().unwrap();

    let home_dir = home::home_dir().ok_or_else(|| eyre!("Failed to get home directory."))?;

    let config_dir = home_dir.join(".stackify");
    std::fs::create_dir_all(&config_dir)?;
    let db_path = config_dir.join("stackifyd.db");

    let mut conn = SqliteConnection::establish(
        db_path
            .to_str()
            .ok_or_else(|| eyre!("Failed to convert database path to string."))?,
    )
    .or_else(|e| bail!("failed to establish database connection: {:?}", e))?;

    apply_db_migrations(&mut conn)?;

    Ok(db_path.display().to_string())
}

/// Applies any pending application database migrations. Initializes the
/// database if it does not already exist.
pub fn apply_db_migrations(conn: &mut SqliteConnection) -> Result<()> {
    conn.batch_execute("
        PRAGMA journal_mode = WAL;          -- better write-concurrency
        PRAGMA synchronous = NORMAL;        -- fsync only in critical moments
        PRAGMA wal_autocheckpoint = 1000;   -- write WAL changes back every 1000 pages, for an in average 1MB WAL file. May affect readers if number is increased
        PRAGMA wal_checkpoint(TRUNCATE);    -- free some space by truncating possibly massive WAL files from the last run.
        PRAGMA busy_timeout = 250;          -- sleep if the database is busy
        PRAGMA foreign_keys = ON;           -- enforce foreign keys
    ")?;

    let has_pending_migrations = MigrationHarness::has_pending_migration(conn, DB_MIGRATIONS)
        .or_else(|e| bail!("failed to determine database migration state: {:?}", e))?;

    if has_pending_migrations {
        info!("there are pending database migrations - updating the database");

        MigrationHarness::run_pending_migrations(conn, DB_MIGRATIONS)
            .or_else(|e| bail!("failed to run database migrations: {:?}", e))?;

        info!("database migrations have been applied successfully");
    }

    Ok(())
}
