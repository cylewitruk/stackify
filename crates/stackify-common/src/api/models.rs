use std::collections::HashMap;

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetStatusResponse {
    pub status: Status,
    pub services: HashMap<Service, ServiceStatus>
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    Ok,
    Degraded,
    Error
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServiceStatus {
    Running,
    Stopped,
    Error
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Service {
    BitcoinMiner = 0,
    BitcoinFollower = 1,
    StacksMiner = 2,
    StacksFollower = 3,
    StacksSigner = 4,
    StacksStackerSelf = 5,
    StacksStackerPool = 6
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateConfigRequest {
    pub service: Service,
    pub config: String
}