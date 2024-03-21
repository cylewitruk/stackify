pub mod stackify_docker;
#[cfg(test)]
pub mod tests;

pub const STACKIFY_DOCKERFILE: &str = include_str!("../../../../assets/Dockerfile");
pub const STACKIFY_CARGO_CONFIG: &str = include_str!("../../../../assets/cargo-config.toml");