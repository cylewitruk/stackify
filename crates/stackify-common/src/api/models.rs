use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::ServiceState;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetStatusResponse {
    pub status: Status,
    pub services: HashMap<Service, ServiceState>
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    Ok,
    Degraded,
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
    StacksStackerPool = 6,
    StackifyEnvironment = 7,
    StackifyDaemon = 8
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateConfigRequest {
    pub service: Service,
    pub config: String
}