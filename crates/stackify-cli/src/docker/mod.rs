use std::path::PathBuf;

use stackify_common::types::EnvironmentName;

pub mod api;
pub mod opts;

pub const STACKIFY_PREFIX: &str = "stx-";

pub struct StackifyHostDirs {
    /// The local directory where Stackify binaries are stored. This includes
    /// built artifacts which are mounted to containers.
    /// Default: `~/.stackify/bin/`.
    pub bin_dir: PathBuf,
    /// The local directory where Stackify temporary data is stored.
    /// Default: `~/.stackify/tmp/`.
    pub tmp_dir: PathBuf,
    /// The local directory where Stackify assets are stored. These are additional
    /// files such as configuration file templates, shell scripts, etc.
    /// Default: `~/.stackify/assets/`.
    pub assets_dir: PathBuf,
}

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

pub fn format_container_name(env_name: &EnvironmentName, container_name: &str) -> String {
    format!("{}{}", STACKIFY_PREFIX, env_name)
}

pub fn format_network_name(env_name: &EnvironmentName) -> String {
    format!("{}{}", STACKIFY_PREFIX, env_name)
}
