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
        cli_name -> Text,
        allow_minimum_epoch -> Bool,
        allow_maximum_epoch -> Bool,
        allow_git_target -> Bool,
    }
}

table! {
    file_type (id) {
        id -> Integer,
        name -> Text,
    }
}

table! {
    service_type_file (id) {
        id -> Integer,
        service_type_id -> Integer,
        file_type_id -> Integer,
        filename -> Text,
        destination_dir -> Text,
        description -> Text,
        default_contents -> Binary,
    }
}

table! {
    service_type_param (id) {
        id -> Integer,
        service_type_id -> Integer,
        name -> Text,
        key -> Text,
        description -> Text,
        default_value -> Nullable<Text>,
        is_required -> Bool,
        value_type_id -> Integer,
        allowed_values -> Nullable<Text>,

    }
}

table! {
    service_type_port (id) {
        id -> Integer,
        service_type_id -> Integer,
        network_protocol_id -> Integer,
        port -> Integer,
        remark -> Nullable<Text>,
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
        cli_name -> Text,
        rebuild_required -> Bool,
        last_built_at -> Nullable<Timestamp>,
        last_build_commit_hash -> Nullable<Text>,
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
    environment_service (id) {
        id -> Integer,
        environment_id -> Integer,
        service_version_id -> Integer,
        name -> Text,
        comment -> Nullable<Text>,
    }
}

table! {
    environment_service_port (id) {
        id -> Integer,
        environment_service_id -> Integer,
        source_port -> Integer,
        publish_port -> Integer,
        network_protocol_id -> Integer,
        remark -> Nullable<Text>,
    }
}

table! {
    environment_service_file (id) {
        id -> Integer,
        environment_id -> Integer,
        environment_service_id -> Integer,
        service_type_file_id -> Integer,
        contents -> Binary,
    }
}

table! {
    environment_service_action (id) {
        id -> Integer,
        environment_service_id -> Integer,
        service_action_type_id -> Integer,
        at_block_height -> Nullable<Integer>,
        at_epoch_id -> Nullable<Integer>,
        data -> Nullable<Text>,
    }
}

table! {
    environment_service_param (id) {
        id -> Integer,
        environment_service_id -> Integer,
        service_type_param_id -> Integer,
        value -> Text,
    }
}

table! {
    environment_container (id) {
        id -> Integer,
        environment_id -> Integer,
        container_id -> Text,
        service_id -> Integer,
        service_version_id -> Integer,
        created_at -> Timestamp,
    }
}

table! {
    environment_container_action_log (id) {
        id -> Integer,
        environment_container_id -> Integer,
        service_action_type_id -> Integer,
        at_block_height -> Integer,
        created_at -> Timestamp,
        data -> Nullable<Text>,
    }
}

table! {
    environment_keychain (id) {
        id -> Integer,
        environment_id -> Integer,
        stx_address -> Text,
        amount -> BigInt,
        mnemonic -> Text,
        private_key -> Text,
        public_key -> Text,
        btc_address -> Text,
        remark -> Nullable<Text>,
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
    service_action_type,
    service_action_type_constraint,
    environment_service_action,
    environment_keychain
);
