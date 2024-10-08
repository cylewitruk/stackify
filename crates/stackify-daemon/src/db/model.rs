use time::PrimitiveDateTime;

use diesel::prelude::*;

use super::schema::*;

#[derive(Queryable, Insertable)]
#[diesel(table_name = log)]
pub struct Log {
    pub id: i32,
    pub message: String,
    pub level: String,
    pub service_id: i32,
    pub timestamp: PrimitiveDateTime,
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = service_state)]
pub struct ServiceState {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = service)]
pub struct Service {
    pub id: i32,
    pub service_type_id: i32,
    pub version: String,
    pub expected_service_state_id: i32,
    pub actual_service_state_id: i32,
    pub is_local: bool,
    pub service_data: Option<String>,
}
