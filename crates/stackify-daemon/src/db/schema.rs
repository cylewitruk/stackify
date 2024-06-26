use diesel::prelude::*;

table! {
    log (id) {
        id -> Integer,
        message -> Text,
        level -> Text,
        service_id -> Integer,
        timestamp -> Timestamp
    }
}

table! {
    service_state (id) {
        id -> Integer,
        name -> Text
    }
}

table! {
    service (service_type_id) {
        id -> Integer,
        service_type_id -> Integer,
        version -> Text,
        expected_service_state_id -> Integer,
        actual_service_state_id -> Integer,
        host -> Nullable<Text>,
        p2p_port -> Nullable<Integer>,
        rpc_port -> Nullable<Integer>,
        rpc_username -> Nullable<Text>,
        rpc_password -> Nullable<Text>,
        is_local -> Bool
    }
}
