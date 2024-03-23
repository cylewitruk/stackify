use diesel::prelude::*;

table! {
    epoch (id) {
        id -> Integer,
        name -> Text,
        default_block_height -> Integer,
    }
}

table! {
    environment_status (id) {
        id -> Integer,
        name -> Text,
    }
}

table! {
    environment (id) {
        id -> Integer,
        name -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        bitcoin_block_speed -> Integer,
    }
}

table! {
    environment_epoch (id) {
        id -> Integer,
        environment_id -> Integer,
        epoch_id -> Integer,
        starts_at_block_height -> Integer,
        ends_at_block_height -> Nullable<Integer>,
    }
}

table! {
    service_type (id) {
        id -> Integer,
        name -> Text,
        allow_minimum_epoch -> Bool,
        allow_maximum_epoch -> Bool,
        allow_git_target -> Bool,
    }
}

table! {
    service_version (id) {
        id -> Integer,
        service_type_id -> Integer,
        version -> Text,
        minimum_epoch_id -> Nullable<Integer>,
        maximum_epoch_id -> Nullable<Integer>,
        git_target -> Nullable<Text>,
    }
}

table! {
    service_upgrade_path (id) {
        id -> Integer,
        name -> Text,
        service_type_id -> Integer,
        from_service_version_id -> Integer,
        to_service_version_id -> Integer,
        minimum_epoch_id -> Integer,
        maximum_epoch_id -> Nullable<Integer>,
    }
}

table! {
    service (id) {
        id -> Integer,
        name -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        environment_id -> Integer,
        service_type_id -> Integer,
        start_at_block_height -> Integer,
        stop_at_block_height -> Nullable<Integer>,
    }
}

table! {
    service_action_type (id) {
        id -> Integer,
        name -> Text,
        requires_running_service -> Bool,
        requires_network -> Bool,
    }
}

table! {
    service_action_type_constraint (id) {
        id -> Integer,
        service_action_id -> Integer,
        allowed_after_service_action_id -> Nullable<Integer>,
    }
}

table! {
    environment_service_action (id) {
        id -> Integer,
        environment_id -> Integer,
        service_id -> Integer,
        service_action_type_id -> Integer,
        at_block_height -> Integer,
        data -> Nullable<Text>,
    }
}

allow_tables_to_appear_in_same_query!(
    epoch,
    environment_status,
    environment,
    environment_epoch,
    service_type,
    service_version,
    service_upgrade_path,
    service,
    service_action_type,
    service_action_type_constraint,
    environment_service_action,
);
