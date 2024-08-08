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

impl From<model::EnvironmentKeychain> for types::EnvironmentKeychain {
    fn from(value: model::EnvironmentKeychain) -> Self {
        types::EnvironmentKeychain {
            id: value.id,
            environment_id: value.environment_id,
            stx_address: value.stx_address,
            private_key: value.private_key,
            public_key: value.public_key,
            amount: value.amount as u64,
            mnemonic: value.mnemonic,
            btc_address: value.btc_address,
            remark: value.remark,
        }
    }
}
