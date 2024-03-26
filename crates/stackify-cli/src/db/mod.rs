#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::HashMap;

use color_eyre::eyre::bail;
use color_eyre::eyre::{Error, Report};
use color_eyre::eyre::Result;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel_migrations::embed_migrations;
use diesel_migrations::EmbeddedMigrations;
use diesel_migrations::MigrationHarness;
use log::info;

pub mod model;
pub mod opts;
pub mod schema;

use self::model::*;
use self::opts::NewServiceVersionOpts;
use self::schema::*;

pub const DB_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

pub struct AppDb {
    conn: RefCell<SqliteConnection>,
}

impl AppDb {
    pub fn new(conn: SqliteConnection) -> Self {
        Self {
            conn: RefCell::new(conn),
        }
    }
}

/// Environments
impl AppDb {
    pub fn get_environment_by_name(&self, name: &str) -> Result<Environment> {
        environment::table
            .filter(environment::name.eq(name))
            .first(&mut *self.conn.borrow_mut())
            .map_err(|e| e.into())
    }

    pub fn update_environment_epochs(&self, epochs: HashMap<i32, i32>) -> Result<()> {
        let conn = &mut *self.conn.borrow_mut();

        conn.transaction(|tx| {
            for (env_epoch_id, height) in epochs {
                diesel::update(environment_epoch::table)
                    .filter(environment_epoch::id.eq(env_epoch_id))
                    .set(environment_epoch::starts_at_block_height.eq(height))
                    .execute(tx)
                    .map_err(Report::from)?;
            }
            Ok::<(), color_eyre::eyre::Error>(())
        })?;

        Ok(())
    }
    
    pub fn list_environments(&self) -> Result<Vec<Environment>> {
        Ok(environment::table
            .order_by(environment::name.asc())
            .load(&mut *self.conn.borrow_mut())?)
    }

    pub fn create_environment(&self, name: &str, bitcoin_block_speed: u32) -> Result<Environment> {
        Ok(diesel::insert_into(environment::table)
            .values((
                environment::name.eq(name),
                environment::bitcoin_block_speed.eq(bitcoin_block_speed as i32),
            ))
            .get_result::<Environment>(&mut *self.conn.borrow_mut())?)
    }

    pub fn delete_environment(&self, name: &str) -> Result<()> {
        let environment_id: Option<i32> = environment::table
            .select(environment::id)
            .filter(environment::name.eq(name))
            .first::<i32>(&mut *self.conn.borrow_mut())
            .optional()?;

        if let Some(environment_id) = environment_id {
            let service_ids = service::table
                .select(service::id)
                .filter(service::environment_id.eq(environment_id))
                .load::<i32>(&mut *self.conn.borrow_mut())?;

            diesel::delete(
                environment_service_action::table
                    .filter(environment_service_action::service_id.eq_any(service_ids)),
            )
            .execute(&mut *self.conn.borrow_mut())?;

            diesel::delete(service::table.filter(service::environment_id.eq(environment_id)))
                .execute(&mut *self.conn.borrow_mut())?;
        }

        Ok(())
    }

    pub fn list_environment_services(&self, name: &str) -> Result<Vec<EnvironmentService>> {
        let environment_id: Option<i32> = environment::table
            .select(environment::id)
            .filter(environment::name.eq(name))
            .first::<i32>(&mut *self.conn.borrow_mut())
            .optional()?;

        if let Some(environment_id) = environment_id {
            Ok(environment_service::table
                .filter(environment_service::environment_id.eq(environment_id))
                .get_results::<EnvironmentService>(&mut *self.conn.borrow_mut())?)
        } else {
            bail!("environment not found")
        }
    }

    pub fn list_environment_epochs(&self, environment_id: i32) -> Result<Vec<EnvironmentEpoch>> {
        Ok(environment_epoch::table
            .filter(environment_epoch::environment_id.eq(environment_id))
            .get_results::<EnvironmentEpoch>(&mut *self.conn.borrow_mut())?)
    }
}

/// Configuration
impl AppDb {
    pub fn list_services(&self) -> Result<Vec<Service>> {
        Ok(service::table
            .order_by(service::name.asc())
            .load(&mut *self.conn.borrow_mut())?)
    }

    pub fn list_service_types(&self) -> Result<Vec<ServiceType>> {
        Ok(service_type::table
            .order_by(service_type::name.asc())
            .load(&mut *self.conn.borrow_mut())?)
    }

    pub fn list_epochs(&self) -> Result<Vec<Epoch>> {
        Ok(epoch::table
            .order_by(epoch::name.asc())
            .load(&mut *self.conn.borrow_mut())?)
    }

    pub fn list_service_versions(&self) -> Result<Vec<ServiceVersion>> {
        Ok(service_version::table
            .order_by(service_version::version.asc())
            .load(&mut *self.conn.borrow_mut())?)
    }

    pub fn list_service_upgrade_paths(&self) -> Result<Vec<ServiceUpgradePath>> {
        Ok(service_upgrade_path::table
            .order_by(service_upgrade_path::name.asc())
            .load(&mut *self.conn.borrow_mut())?)
    }

    pub fn new_service_version(&self, opts: NewServiceVersionOpts) -> Result<ServiceVersion> {
        Ok(diesel::insert_into(service_version::table)
            .values((
                service_version::service_type_id.eq(opts.service_type_id),
                service_version::version.eq(&opts.version),
                service_version::cli_name.eq(&opts.cli_name),
                service_version::git_target.eq(opts.git_target.as_deref()),
                service_version::minimum_epoch_id.eq(opts.minimum_epoch_id),
                service_version::maximum_epoch_id.eq(opts.maximum_epoch_id),
            ))
            .get_result::<ServiceVersion>(&mut *self.conn.borrow_mut())?)
    }

    pub fn new_epoch(&self, name: &str, default_block_height: u32) -> Result<Epoch> {
        Ok(diesel::insert_into(epoch::table)
            .values((
                epoch::name.eq(name),
                epoch::default_block_height.eq(default_block_height as i32),
            ))
            .get_result::<Epoch>(&mut *self.conn.borrow_mut())?)
    }
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
