use diesel::prelude::*;

table! {
    epoch (id) {
        id -> Integer,
        name -> Text,
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
    service_type (id) {
        id -> Integer,
        name -> Text,
        minimum_epoch_id -> Nullable<Integer>,
        maximum_epoch_id -> Nullable<Integer>,
    }
}

table! {
    service_version (id) {
        id -> Integer,
        service_type_id -> Integer,
        version -> Text,
        minimum_epoch_id -> Nullable<Integer>,
        maximum_epoch_id -> Nullable<Integer>,
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
    service_upgrade (id) {
        id -> Integer,
        service_id -> Integer,
        service_upgrade_path_id -> Integer,
        at_block_height -> Integer,
    }
}