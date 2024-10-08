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

#[derive(Queryable, PartialEq, Debug, Selectable)]
#[diesel(table_name = service_type_file)]
pub struct ServiceTypeFileHeader {
    pub id: i32,
    pub service_type_id: i32,
    pub file_type_id: i32,
    pub filename: String,
    pub destination_dir: String,
    pub description: String,
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
#[diesel(table_name = service_type_port)]
pub struct ServiceTypePort {
    pub id: i32,
    pub service_type_id: i32,
    pub network_protocol_id: i32,
    pub port: i32,
    pub remark: Option<String>,
}

// TODO: Change this to `ServiceTypeVersion` like the other models which aren't directly connected to an environment.
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
    pub rebuild_required: bool,
    pub last_built_at: Option<PrimitiveDateTime>,
    pub last_build_commit_hash: Option<String>,
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

/*table! {
    environment_service_port (id) {
        id -> Integer,
        environment_service_id -> Integer,
        source_port -> Integer,
        publish_port -> Integer,
        network_protocol_id -> Integer,
        remark -> Nullable<Text>,
    }
} */
#[derive(Queryable, Selectable, Identifiable, PartialEq, Eq, Debug, Clone, QueryableByName)]
#[diesel(table_name = environment_service_port)]
pub struct EnvironmentServicePort {
    pub id: i32,
    pub environment_service_id: i32,
    pub source_port: i32,
    pub publish_port: i32,
    pub network_protocol_id: i32,
    pub remark: Option<String>,
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
#[diesel(table_name = environment_service_param)]
pub struct EnvironmentServiceParam {
    pub id: i32,
    pub environment_service_id: i32,
    pub service_type_param_id: i32,
    pub value: String,
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

#[derive(
    Queryable, Associations, Selectable, Identifiable, PartialEq, Eq, Debug, Clone, QueryableByName,
)]
#[diesel(table_name = environment_container_action_log)]
#[belongs_to(EnvironmentContainer)]
pub struct EnvironmentContainerActionLog {
    pub id: i32,
    pub environment_container_id: i32,
    pub service_action_type_id: i32,
    pub at_block_height: i32,
    pub created_at: PrimitiveDateTime,
    pub data: Option<String>,
}

#[derive(
    Queryable, Associations, Selectable, Identifiable, PartialEq, Eq, Debug, Clone, QueryableByName,
)]
#[diesel(table_name = environment_keychain)]
#[belongs_to(Environment)]
pub struct EnvironmentKeychain {
    pub id: i32,
    pub environment_id: i32,
    pub stx_address: String,
    pub amount: i64,
    pub mnemonic: String,
    pub private_key: String,
    pub public_key: String,
    pub btc_address: String,
    pub remark: Option<String>,
}
