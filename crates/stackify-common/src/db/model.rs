use time::PrimitiveDateTime;
use diesel::prelude::*;
use super::schema::*;

#[derive(
    Queryable, Selectable, Identifiable, PartialEq, Eq, Debug, Clone, 
    QueryableByName
)]
#[diesel(table_name = epoch)]
pub struct Epoch {
    pub id: i32,
    pub name: String,
}

#[derive(
    Queryable, Selectable, Identifiable, PartialEq, Eq, Debug, Clone, 
    QueryableByName
)]
#[diesel(table_name = environment_status)]
pub struct EnvironmentStatus {
    pub id: i32,
    pub name: String,
}

#[derive(
    Queryable, Selectable, Identifiable, PartialEq, Eq, Debug, Clone, 
    QueryableByName, Insertable,
)]
#[diesel(table_name = environment)]
pub struct Environment {
    pub id: i32,
    pub name: String,
    pub created_at: PrimitiveDateTime,
    pub updated_at: PrimitiveDateTime,
    pub bitcoin_block_speed: i32,
}

#[derive(
    Queryable, Selectable, Identifiable, PartialEq, Eq, Debug, Clone, 
    QueryableByName
)]
#[diesel(table_name = service_type)]
pub struct ServiceType {
    pub id: i32,
    pub name: String,
    pub minimum_epoch_id: Option<i32>,
    pub maximum_epoch_id: Option<i32>,
}

#[derive(
    Queryable, Selectable, Identifiable, PartialEq, Eq, Debug, Clone, 
    QueryableByName
)]
#[diesel(table_name = service_version)]
pub struct ServiceVersion {
    pub id: i32,
    pub service_type_id: i32,
    pub version: String,
    pub minimum_epoch_id: Option<i32>,
    pub maximum_epoch_id: Option<i32>,
}

#[derive(
    Queryable, Selectable, Identifiable, PartialEq, Eq, Debug, Clone, 
    QueryableByName
)]
#[diesel(table_name = service_upgrade_path)]
pub struct ServiceUpgradePath {
    pub id: i32,
    pub name: String,
    pub service_type_id: i32,
    pub from_service_version_id: i32,
    pub to_service_version_id: i32,
    pub minimum_epoch_id: i32,
    pub maximum_epoch_id: Option<i32>,
}

#[derive(
    Queryable, Selectable, Identifiable, PartialEq, Eq, Debug, Clone, 
    QueryableByName, Insertable,
)]
#[diesel(table_name = service)]
pub struct Service {
    pub id: i32,
    pub name: String,
    pub created_at: PrimitiveDateTime,
    pub updated_at: PrimitiveDateTime,
    pub environment_id: i32,
    pub service_type_id: i32,
    pub start_at_block_height: i32,
    pub stop_at_block_height: Option<i32>,
}

#[derive(
    Queryable, Selectable, Identifiable, PartialEq, Eq, Debug, Clone, 
    QueryableByName, Insertable,
)]
#[diesel(table_name = service_upgrade)]
pub struct ServiceUpgrade {
    pub id: i32,
    pub service_id: i32,
    pub service_upgrade_path_id: i32,
    pub at_block_height: i32,
}