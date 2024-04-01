use super::schema::*;
use diesel::prelude::*;
use time::PrimitiveDateTime;

#[derive(Queryable, Selectable, Identifiable, PartialEq, Eq, Debug, Clone, QueryableByName)]
#[diesel(table_name = epoch)]
pub struct Epoch {
    pub id: i32,
    pub name: String,
    pub default_block_height: i32,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Eq, Debug, Clone, QueryableByName)]
#[diesel(table_name = environment_status)]
pub struct EnvironmentStatus {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Eq, Debug, Clone, QueryableByName)]
#[diesel(table_name = environment)]
pub struct Environment {
    pub id: i32,
    pub name: String,
    pub created_at: PrimitiveDateTime,
    pub updated_at: PrimitiveDateTime,
    pub bitcoin_block_speed: i32,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Eq, Debug, Clone, QueryableByName)]
#[diesel(table_name = environment_epoch)]
pub struct EnvironmentEpoch {
    pub id: i32,
    pub environment_id: i32,
    pub epoch_id: i32,
    pub starts_at_block_height: i32,
    pub ends_at_block_height: Option<i32>,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Eq, Debug, Clone, QueryableByName)]
#[diesel(table_name = service_type)]
pub struct ServiceType {
    pub id: i32,
    pub name: String,
    pub cli_name: String,
    pub allow_minimum_epoch: bool,
    pub allow_maximum_epoch: bool,
    pub allow_git_target: bool,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Eq, Debug, Clone, QueryableByName)]
#[diesel(table_name = file_type)]
pub struct FileType {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Eq, Debug, Clone, QueryableByName)]
#[diesel(table_name = service_type_file)]
pub struct ServiceTypeFile {
    pub id: i32,
    pub service_type_id: i32,
    pub file_type_id: i32,
    pub filename: String,
    pub destination_dir: String,
    pub description: String,
    pub default_contents: Vec<u8>,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Eq, Debug, Clone, QueryableByName)]
#[diesel(table_name = service_type_param)]
pub struct ServiceTypeParam {
    pub id: i32,
    pub service_type_id: i32,
    pub name: String,
    pub key: String,
    pub description: String,
    pub default_value: Option<String>,
    pub is_required: bool,
    pub value_type_id: i32,
    pub allowed_values: Option<String>,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Eq, Debug, Clone, QueryableByName)]
#[diesel(table_name = service_version)]
pub struct ServiceVersion {
    pub id: i32,
    pub service_type_id: i32,
    pub version: String,
    pub minimum_epoch_id: Option<i32>,
    pub maximum_epoch_id: Option<i32>,
    pub git_target: Option<String>,
    pub cli_name: String,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Eq, Debug, Clone, QueryableByName)]
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

#[derive(Queryable, Selectable, Identifiable, PartialEq, Eq, Debug, Clone, QueryableByName)]
#[diesel(table_name = service_action_type)]
pub struct ServiceActionType {
    pub id: i32,
    pub name: String,
    pub requires_running_service: bool,
    pub requires_network: bool,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Eq, Debug, Clone, QueryableByName)]
#[diesel(table_name = service_action_type_constraint)]
pub struct ServiceActionTypeConstraint {
    pub id: i32,
    pub service_action_id: i32,
    pub allowed_after_service_action_id: Option<i32>,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Eq, Debug, Clone, QueryableByName)]
#[diesel(table_name = environment_service)]
pub struct EnvironmentService {
    pub id: i32,
    pub environment_id: i32,
    pub service_version_id: i32,
    pub name: String,
    pub comment: Option<String>,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Eq, Debug, Clone, QueryableByName)]
#[diesel(table_name = environment_service_file)]
pub struct EnvironmentServiceFile {
    pub id: i32,
    pub environment_id: i32,
    pub environment_service_id: i32,
    pub service_type_file_id: i32,
    pub contents: Vec<u8>,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Eq, Debug, Clone, QueryableByName)]
#[diesel(table_name = environment_service_action)]
pub struct EnvironmentServiceAction {
    pub id: i32,
    pub environment_service_id: i32,
    pub service_action_type_id: i32,
    pub at_block_height: Option<i32>,
    pub at_epoch_id: Option<i32>,
    pub data: Option<String>,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Eq, Debug, Clone, QueryableByName)]
#[diesel(table_name = environment_container)]
pub struct EnvironmentContainer {
    pub id: i32,
    pub environment_id: i32,
    pub container_id: String,
    pub service_id: i32,
    pub service_version_id: i32,
    pub created_at: PrimitiveDateTime,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Eq, Debug, Clone, QueryableByName)]
#[diesel(table_name = environment_container_action_log)]
pub struct EnvironmentContainerActionLog {
    pub id: i32,
    pub environment_container_id: i32,
    pub service_action_type_id: i32,
    pub at_block_height: i32,
    pub created_at: PrimitiveDateTime,
    pub data: Option<String>,
}
