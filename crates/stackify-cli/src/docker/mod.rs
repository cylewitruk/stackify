use std::{
    fmt::{Display, Formatter},
    ops::Deref,
    path::PathBuf,
};

use color_eyre::{eyre::eyre, Result};
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

#[derive(Debug, Clone)]
pub struct ContainerUser {
    pub uid: u32,
    pub gid: u32,
}

impl ContainerUser {
    pub fn new(uid: u32, gid: u32) -> Self {
        Self { uid, gid }
    }

    pub fn root() -> Self {
        Self::new(0, 0)
    }
}

impl Display for ContainerUser {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}:{}", self.uid, self.gid)
    }
}

pub fn format_environment_container_name(env_name: &EnvironmentName) -> String {
    format!("{}{}", STACKIFY_PREFIX, env_name)
}

pub fn format_service_container_name(service_name: &str) -> String {
    format!("{}{}", STACKIFY_PREFIX, service_name)
}

pub fn format_network_name(env_name: &EnvironmentName) -> String {
    format!("{}{}", STACKIFY_PREFIX, env_name)
}

pub enum ActionResult {
    Success,
    Failed(i64, Vec<String>),
    Cancelled,
}

pub enum BuildResult {
    Success(String),
    Failed(String, String),
    Cancelled,
}

#[derive(Debug)]
pub enum LabelKey {
    Stackify,
    EnvironmentName,
    ServiceType,
    ServiceVersion,
    IsLeader,
    ServiceId,
}

impl std::fmt::Display for LabelKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.deref())
    }
}

impl Deref for LabelKey {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            LabelKey::Stackify => "local.stackify",
            LabelKey::EnvironmentName => "local.stackify.environment",
            LabelKey::ServiceType => "local.stackify.service_type",
            LabelKey::ServiceVersion => "local.stackify.service_version",
            LabelKey::IsLeader => "local.stackify.is_leader",
            LabelKey::ServiceId => "local.stackify.service_id",
        }
    }
}

impl Into<String> for LabelKey {
    fn into(self) -> String {
        self.to_string()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ContainerState {
    Created,
    Running,
    Paused,
    Restarting,
    Removing,
    Exited,
    Dead,
}

impl ContainerState {
    pub fn parse(s: &str) -> Result<ContainerState> {
        let state = match s {
            "created" => ContainerState::Created,
            "running" => ContainerState::Running,
            "paused" => ContainerState::Paused,
            "restarting" => ContainerState::Restarting,
            "removing" => ContainerState::Removing,
            "exited" => ContainerState::Exited,
            "dead" => ContainerState::Dead,
            _ => {
                return Err(eyre!("Unknown container state: {}", s));
            }
        };
        Ok(state)
    }
}
