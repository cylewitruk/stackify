#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::HashMap;

use ::diesel::connection::SimpleConnection;
use ::diesel::prelude::*;
use ::diesel::upsert::excluded;
use ::diesel::{delete, insert_into, update};
use color_eyre::eyre::bail;
use color_eyre::eyre::Report;
use color_eyre::eyre::Result;
use diesel_migrations::embed_migrations;
use diesel_migrations::EmbeddedMigrations;
use diesel_migrations::MigrationHarness;
use log::info;
use stackify_common::ValueType;

pub mod cli_db;
pub mod diesel;
pub mod errors;
pub mod opts;

#[cfg(test)]
mod tests;

use self::diesel::model::*;
use self::diesel::schema::*;
use self::opts::NewServiceVersionOpts;

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
    pub fn set_service_param_value(
        &self,
        env_svc_id: i32,
        param_id: i32,
        value: String,
    ) -> Result<()> {
        insert_into(environment_service_param::table)
            .values((
                environment_service_param::environment_service_id.eq(env_svc_id),
                environment_service_param::service_type_param_id.eq(param_id),
                environment_service_param::value.eq(&value),
            ))
            .on_conflict((
                environment_service_param::environment_service_id,
                environment_service_param::service_type_param_id,
            ))
            .do_update()
            .set(environment_service_param::value.eq(&value))
            .execute(&mut *self.conn.borrow_mut())
            .map(|_| ())
            .map_err(Report::from)?;

        Ok(())
    }

    pub fn update_service_version_build_details(
        &self,
        service_version_id: i32,
        commit_hash: Option<&str>,
    ) -> Result<()> {
        ::diesel::update(service_version::table)
            .filter(service_version::id.eq(service_version_id))
            .set((
                service_version::last_built_at.eq(::diesel::dsl::now),
                service_version::last_build_commit_hash.eq(commit_hash),
            ))
            .execute(&mut *self.conn.borrow_mut())
            .map(|_| ())?;

        Ok(())
    }

    pub fn add_environment_keychain(
        &self,
        environment_id: i32,
        stx_address: &str,
        btc_address: &str,
        public_key: &str,
        private_key: &str,
        mnemonic: &str,
        balance: u64,
        remark: &str
    ) -> Result<EnvironmentKeychain> {
        Ok(insert_into(environment_keychain::table)
            .values((
                environment_keychain::environment_id.eq(environment_id),
                environment_keychain::stx_address.eq(stx_address),
                environment_keychain::btc_address.eq(btc_address),
                environment_keychain::public_key.eq(public_key),
                environment_keychain::private_key.eq(private_key),
                environment_keychain::mnemonic.eq(mnemonic),
                environment_keychain::amount.eq(balance as i64),
                environment_keychain::remark.eq(remark),
            ))
            .get_result(&mut *self.conn.borrow_mut())?)
    }

    pub fn add_environment_service(
        &self,
        environment_id: i32,
        service_version_id: i32,
        name: &str,
        comment: Option<&str>,
    ) -> Result<EnvironmentService> {
        Ok(insert_into(environment_service::table)
            .values((
                environment_service::environment_id.eq(environment_id),
                environment_service::service_version_id.eq(service_version_id),
                environment_service::name.eq(name),
                environment_service::comment.eq(comment),
            ))
            .get_result(&mut *self.conn.borrow_mut())?)
    }

    pub fn add_environment_service_action(
        &self,
        environment_service_id: i32,
        service_action_type_id: i32,
        at_block_height: Option<i32>,
        at_epoch_id: Option<i32>,
    ) -> Result<()> {
        insert_into(environment_service_action::table)
            .values((
                environment_service_action::environment_service_id.eq(environment_service_id),
                environment_service_action::service_action_type_id.eq(service_action_type_id),
                environment_service_action::at_block_height.eq(at_block_height),
                environment_service_action::at_epoch_id.eq(at_epoch_id),
            ))
            .execute(&mut *self.conn.borrow_mut())?;

        Ok(())
    }

    pub fn add_environment_service_file(
        &self,
        environment_id: i32,
        environment_service_id: i32,
        service_type_file_id: i32,
        contents: &[u8],
    ) -> Result<()> {
        insert_into(environment_service_file::table)
            .values((
                environment_service_file::environment_id.eq(environment_id),
                environment_service_file::environment_service_id.eq(environment_service_id),
                environment_service_file::service_type_file_id.eq(service_type_file_id),
                environment_service_file::contents.eq(contents),
            ))
            .execute(&mut *self.conn.borrow_mut())?;

        Ok(())
    }

    pub fn add_environment_service_param(
        &self,
        environment_service_id: i32,
        service_type_param_id: i32,
        value: &str,
    ) -> Result<()> {
        insert_into(environment_service_param::table)
            .values((
                environment_service_param::environment_service_id.eq(environment_service_id),
                environment_service_param::service_type_param_id.eq(service_type_param_id),
                environment_service_param::value.eq(value),
            ))
            .execute(&mut *self.conn.borrow_mut())?;

        Ok(())
    }

    pub fn find_service_type_param_id_by_key(
        &self,
        service_type_id: i32,
        key: &str,
    ) -> Result<i32> {
        Ok(service_type_param::table
            .select(service_type_param::id)
            .filter(service_type_param::service_type_id.eq(service_type_id))
            .filter(service_type_param::key.eq(key))
            .first(&mut *self.conn.borrow_mut())?)
    }

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
                update(environment_epoch::table)
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
        let conn = &mut *self.conn.borrow_mut();

        conn.transaction(|tx| {
            let env = insert_into(environment::table)
                .values((
                    environment::name.eq(name),
                    environment::bitcoin_block_speed.eq(bitcoin_block_speed as i32),
                ))
                .get_result::<Environment>(tx)?;

            let epochs = epoch::table.order_by(epoch::id.asc()).load::<Epoch>(tx)?;

            for epoch in epochs {
                insert_into(environment_epoch::table)
                    .values((
                        environment_epoch::environment_id.eq(env.id),
                        environment_epoch::epoch_id.eq(epoch.id),
                        environment_epoch::starts_at_block_height.eq(epoch.default_block_height),
                    ))
                    .execute(tx)?;
            }

            Ok(env)
        })
    }

    pub fn delete_environment(&self, name: &str) -> Result<()> {
        let environment_id: Option<i32> = environment::table
            .select(environment::id)
            .filter(environment::name.eq(name))
            .first::<i32>(&mut *self.conn.borrow_mut())
            .optional()?;

        if let Some(environment_id) = environment_id {
            let environment_service_ids = environment_service::table
                .select(environment_service::id)
                .filter(environment_service::environment_id.eq(environment_id))
                .load::<i32>(&mut *self.conn.borrow_mut())?;

            delete(
                environment_service_action::table
                    .filter(environment_service_action::id.eq_any(environment_service_ids)),
            )
            .execute(&mut *self.conn.borrow_mut())?;

            delete(
                environment_service::table
                    .filter(environment_service::environment_id.eq(environment_id)),
            )
            .execute(&mut *self.conn.borrow_mut())?;
        }

        Ok(())
    }

    pub fn list_environment_services_for_environment_id(
        &self,
        environment_id: i32,
    ) -> Result<Vec<EnvironmentService>> {
        Ok(environment_service::table
            .filter(environment_service::environment_id.eq(environment_id))
            .get_results::<EnvironmentService>(&mut *self.conn.borrow_mut())?)
    }

    pub fn list_environment_services(&self) -> Result<Vec<EnvironmentService>> {
        Ok(environment_service::table
            .order_by(environment_service::name.asc())
            .load(&mut *self.conn.borrow_mut())?)
    }

    pub fn list_environment_service_files(
        &self,
        environment_service_id: i32,
    ) -> Result<Vec<EnvironmentServiceFile>> {
        Ok(environment_service_file::table
            .filter(environment_service_file::environment_service_id.eq(environment_service_id))
            .get_results::<EnvironmentServiceFile>(&mut *self.conn.borrow_mut())?)
    }

    pub fn list_environment_epochs(&self, environment_id: i32) -> Result<Vec<EnvironmentEpoch>> {
        Ok(environment_epoch::table
            .filter(environment_epoch::environment_id.eq(environment_id))
            .get_results::<EnvironmentEpoch>(&mut *self.conn.borrow_mut())?)
    }
}

pub struct InsertServiceFile {
    pub service_type_id: i32,
    pub file_type_id: i32,
    /// The filename of the file as it will be copied into the Docker container.
    pub filename: String,
    /// The destination path within the service's Docker container,
    /// _excluding filename_.
    pub destination_dir: String,
    pub description: String,
    pub default_contents: Vec<u8>,
}

pub struct InsertServiceParam<'a> {
    pub service_type: &'a stackify_common::ServiceType,
    pub name: &'a str,
    pub key: &'a str,
    pub description: &'a str,
    pub default_value: Option<&'a str>,
    pub is_required: bool,
    pub value_type: &'a ValueType,
    pub allowed_values: Option<&'a str>,
}

/// Configuration
impl AppDb {
    pub fn list_service_type_params_for_service_type(
        &self,
        service_type_id: i32,
    ) -> Result<Vec<ServiceTypeParam>> {
        Ok(service_type_param::table
            .filter(service_type_param::service_type_id.eq(service_type_id))
            .load(&mut *self.conn.borrow_mut())?)
    }
    pub fn check_if_service_type_file_exists(
        &self,
        service_type_id: i32,
        filename: &str,
    ) -> Result<bool> {
        let count: i64 = service_type_file::table
            .filter(service_type_file::service_type_id.eq(service_type_id))
            .filter(service_type_file::filename.eq(filename))
            .count()
            .get_result(&mut *self.conn.borrow_mut())?;

        Ok(count > 0)
    }

    pub fn check_if_service_type_param_exists(
        &self,
        service_type_id: i32,
        key: &str,
    ) -> Result<bool> {
        let count: i64 = service_type_param::table
            .filter(service_type_param::service_type_id.eq(service_type_id))
            .filter(service_type_param::key.eq(key))
            .count()
            .get_result(&mut *self.conn.borrow_mut())?;

        Ok(count > 0)
    }

    pub fn list_service_files_for_service_type(
        &self,
        service_type_id: i32,
    ) -> Result<Vec<ServiceTypeFile>> {
        Ok(service_type_file::table
            .filter(service_type_file::service_type_id.eq(service_type_id))
            .load(&mut *self.conn.borrow_mut())?)
    }

    pub fn insert_service_file(&self, insert: InsertServiceFile) -> Result<()> {
        insert_into(service_type_file::table)
            .values((
                service_type_file::service_type_id.eq(insert.service_type_id),
                service_type_file::filename.eq(&insert.filename),
                service_type_file::file_type_id.eq(insert.file_type_id),
                service_type_file::destination_dir.eq(&insert.destination_dir),
                service_type_file::description.eq(&insert.description),
                service_type_file::default_contents.eq(&insert.default_contents),
            ))
            .on_conflict((
                service_type_file::service_type_id,
                service_type_file::filename,
            ))
            .do_update()
            .set((
                service_type_file::file_type_id.eq(insert.file_type_id),
                service_type_file::destination_dir.eq(&insert.destination_dir),
                service_type_file::description.eq(&insert.description),
                service_type_file::default_contents.eq(&insert.default_contents),
            ))
            .execute(&mut *self.conn.borrow_mut())?;

        Ok(())
    }

    pub fn insert_service_param(&self, insert: &InsertServiceParam) -> Result<()> {
        let value_type_id = insert.value_type.clone() as i32;
        insert_into(service_type_param::table)
            .values((
                service_type_param::service_type_id.eq(insert.service_type.clone() as i32),
                service_type_param::name.eq(insert.name),
                service_type_param::key.eq(insert.key),
                service_type_param::description.eq(insert.description),
                service_type_param::default_value.eq(insert.default_value),
                service_type_param::is_required.eq(insert.is_required),
                service_type_param::value_type_id.eq(value_type_id),
                service_type_param::allowed_values.eq(insert.allowed_values),
            ))
            .on_conflict((service_type_param::service_type_id, service_type_param::key))
            .do_update()
            .set((
                service_type_param::name.eq(insert.name),
                service_type_param::description.eq(insert.description),
                service_type_param::default_value.eq(insert.default_value),
                service_type_param::is_required.eq(insert.is_required),
                service_type_param::value_type_id.eq(value_type_id),
                service_type_param::allowed_values.eq(insert.allowed_values),
            ))
            .execute(&mut *self.conn.borrow_mut())?;

        Ok(())
    }

    pub fn list_service_types(&self) -> Result<Vec<ServiceType>> {
        Ok(service_type::table
            .order_by(service_type::name.asc())
            .load(&mut *self.conn.borrow_mut())?)
    }

    pub fn list_service_type_files(&self) -> Result<Vec<ServiceTypeFile>> {
        Ok(service_type_file::table.load(&mut *self.conn.borrow_mut())?)
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
        Ok(insert_into(service_version::table)
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
        Ok(insert_into(epoch::table)
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
