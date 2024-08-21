use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinNodeData {
    pub host: String,
    pub rpc_port: u16,
    pub rpc_username: String,
    pub rpc_password: String,
}
