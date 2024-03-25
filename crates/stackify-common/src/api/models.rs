use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::{ServiceState, ServiceType};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetStatusResponse {
    pub status: Status,
    pub services: HashMap<ServiceType, ServiceState>
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    Ok,
    Degraded,
    Error
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateConfigRequest {
    pub service: ServiceType,
    pub config: String
}