use std::path::PathBuf;

use stackify_common::types::EnvironmentName;

pub mod api;
pub mod opts;

pub const STACKIFY_PREFIX: &str = "stx-";

#[derive(Clone, Debug)]
pub struct StackifyContainerDirs {
    /// The home directory of the 'stackify' user within the container.
    /// For example: `/home/stackify/`.
    pub home_dir: PathBuf,
    /// The directory where the 'stackify' user's binaries are stored.
    /// For example: `/home/stackify/bin/`.
    pub bin_dir: PathBuf,
    /// The directory where the 'stackify' user's data is stored.
    /// For example: `/home/stackify/data/`.
    pub data_dir: PathBuf,
    /// The directory where the 'stackify' user's configuration files are stored.
    /// For example: `/home/stackify/config/`.
    pub config_dir: PathBuf,
    /// The directory where the 'stackify' user's logs are stored.
    /// For example: `/home/stackify/logs/`.
    pub logs_dir: PathBuf,
}

impl Default for StackifyContainerDirs {
    fn default() -> Self {
        Self {
            home_dir: PathBuf::from("/home/stackify"),
            bin_dir: PathBuf::from("/home/stackify/bin"),
            data_dir: PathBuf::from("/home/stackify/data"),
            config_dir: PathBuf::from("/home/stackify/config"),
            logs_dir: PathBuf::from("/home/stackify/logs"),
        }
    }
}

pub fn format_container_name(env_name: &EnvironmentName, container_name: &str) -> String {
    format!("{}{}", STACKIFY_PREFIX, env_name)
}

pub fn format_network_name(env_name: &EnvironmentName) -> String {
    format!("{}{}", STACKIFY_PREFIX, env_name)
}
