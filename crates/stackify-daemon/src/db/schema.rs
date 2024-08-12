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
        is_local -> Bool,
        service_data -> Nullable<Text>,
    }
}
