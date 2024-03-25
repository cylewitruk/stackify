use color_eyre::Result;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};
use tokio::sync::RwLock;

pub mod model;
pub mod schema;

/// Application database for the Stackify daemon.
pub struct DaemonDb {
    conn: RwLock<SqliteConnection>,
}

impl DaemonDb {
    pub fn new(conn: SqliteConnection) -> Self {
        Self {
            conn: RwLock::new(conn),
        }
    }
}

impl DaemonDb {
    /// Inserts a new log entry into the local database.
    pub fn insert_log_entry(
        &mut self,
        service_id: i32,
        level: &str,
        message: &str,
    ) -> Result<model::Log> {
        Ok(diesel::insert_into(schema::log::table)
            .values((
                schema::log::message.eq(message),
                schema::log::level.eq(level),
                schema::log::service_id.eq(service_id),
            ))
            .get_result::<model::Log>(&mut *self.conn.get_mut())?)
    }

    /// Retrieves ALL log entries.
    pub fn get_all_log_entries(&mut self) -> Result<Vec<model::Log>> {
        Ok(schema::log::table.load::<model::Log>(&mut *self.conn.get_mut())?)
    }

    /// Retrieves all log entries AFTER a given ID. The id is exclusive, i.e. the
    /// provided log entry id will not be returned, only log entries with a higher id.
    pub fn get_log_entries_after(&mut self, id: i32) -> Result<Vec<model::Log>> {
        Ok(schema::log::table
            .filter(schema::log::id.gt(id))
            .load::<model::Log>(&mut *self.conn.get_mut())?)
    }

    /// Lists the services which this node is responsible for.
    pub fn list_services(&mut self) -> Result<Vec<model::Service>> {
        Ok(schema::service::table.load::<model::Service>(&mut *self.conn.get_mut())?)
    }

    pub fn set_service_state(&mut self, service_id: i32, state_id: i32) -> Result<model::Service> {
        Ok(diesel::update(schema::service::table)
            .filter(schema::service::id.eq(service_id))
            .set(schema::service::actual_service_state_id.eq(state_id))
            .get_result::<model::Service>(&mut *self.conn.get_mut())?)
    }
}
