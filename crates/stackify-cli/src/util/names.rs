use stackify_common::types::{EnvironmentName, EnvironmentService};

pub fn environment_container_name(env_name: &EnvironmentName) -> String {
    format!("/stx-{}", env_name)
}

pub fn service_container_name(env_service: &EnvironmentService) -> String {
    format!("/stx-{}", env_service.name)
}
