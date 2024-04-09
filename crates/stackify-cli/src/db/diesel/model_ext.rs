use super::model;
use stackify_common::types;

impl From<model::ServiceType> for types::ServiceTypeSimple {
    fn from(value: model::ServiceType) -> Self {
        types::ServiceTypeSimple {
            id: value.id,
            name: value.name,
            cli_name: value.cli_name,
        }
    }
}

impl From<model::Epoch> for types::Epoch {
    fn from(value: model::Epoch) -> Self {
        types::Epoch {
            id: value.id,
            name: value.name,
            default_block_height: value.default_block_height as u32,
        }
    }
}
