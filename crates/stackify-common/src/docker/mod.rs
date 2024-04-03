use std::{collections::HashMap, ops::Deref, path::Path};

use color_eyre::eyre::{eyre, Result};

use crate::types::EnvironmentName;

pub mod containers;
pub mod files;
pub mod images;
pub mod networks;
pub mod stackify_docker;
#[cfg(test)]
pub mod tests;
pub mod util;

// pub const STACKIFY_BUILD_DOCKERFILE: &str = include_str!("../../../../assets/Dockerfile.build");
// pub const STACKIFY_RUN_DOCKERFILE: &str = include_str!("../../../../assets/Dockerfile.runtime");
// pub const STACKIFY_CARGO_CONFIG: &str = include_str!("../../../../assets/cargo-config.toml");

#[derive(Debug)]
pub struct NewStacksNetworkResult {
    pub id: String,
    pub name: String,
}

#[derive(Debug)]
pub struct NewStacksNodeContainer<'a> {
    _environment_name: &'a EnvironmentName,
}

pub enum ContainerService {
    Environment,
    BitcoinMiner,
    StacksMiner,
    StacksFollower,
    StacksSigner,
    StacksSelfStacker,
    StacksPoolStacker,
}

impl std::fmt::Display for ContainerService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContainerService::Environment => write!(f, "{}", "Environment"),
            ContainerService::BitcoinMiner => write!(f, "{}", "Bitcoin Miner"),
            ContainerService::StacksMiner => write!(f, "{}", "Stacks Miner"),
            ContainerService::StacksFollower => write!(f, "{}", "Stacks Follower"),
            ContainerService::StacksSigner => write!(f, "{}", "Stacks Signer"),
            ContainerService::StacksSelfStacker => write!(f, "{}", "Stacks Stacker (Self)"),
            ContainerService::StacksPoolStacker => write!(f, "{}", "Stacks Stacker (Pool)"),
        }
    }
}

impl ContainerService {
    pub fn from_id(id: u32) -> Option<Self> {
        match id {
            0 => Some(Self::BitcoinMiner),
            1 => Some(Self::StacksMiner),
            2 => Some(Self::StacksFollower),
            3 => Some(Self::StacksSigner),
            4 => Some(Self::StacksSelfStacker),
            5 => Some(Self::StacksPoolStacker),
            _ => None,
        }
    }

    pub fn id(&self) -> u32 {
        match self {
            Self::Environment => 99,
            Self::BitcoinMiner => 0,
            Self::StacksMiner => 1,
            Self::StacksFollower => 2,
            Self::StacksSigner => 3,
            Self::StacksSelfStacker => 4,
            Self::StacksPoolStacker => 5,
        }
    }

    pub fn to_label_string(&self) -> String {
        match self {
            Self::Environment => "environment".to_string(),
            Self::BitcoinMiner => "bitcoin-miner".to_string(),
            Self::StacksMiner => "stacks-miner".to_string(),
            Self::StacksFollower => "stacks-follower".to_string(),
            Self::StacksSigner => "stacks-signer".to_string(),
            Self::StacksSelfStacker => "stacks-self-stacker".to_string(),
            Self::StacksPoolStacker => "stacks-pool-stacker".to_string(),
        }
    }
}

/*INSERT INTO service_type (id, name) VALUES (0, 'Bitcoin Miner');
INSERT INTO service_type (id, name) VALUES (1, 'Bitcoin Follower');
INSERT INTO service_type (id, name) VALUES (2, 'Stacks Miner');
INSERT INTO service_type (id, name) VALUES (3, 'Stacks Follower');
INSERT INTO service_type (id, name) VALUES (4, 'Stacks Signer'); -- Minimum epoch 2.5
INSERT INTO service_type (id, name) VALUES (5, 'Stacks Stacker (Self)');
INSERT INTO service_type (id, name) VALUES (6, 'Stacks Stacker (Pool)'); */

#[derive(Debug)]
pub enum LabelKey {
    Stackify,
    EnvironmentName,
    Service,
    NodeVersion,
    IsLeader,
}

impl std::fmt::Display for LabelKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LabelKey::Stackify => write!(f, "{}", "local.stackify"),
            LabelKey::EnvironmentName => write!(f, "{}", "local.stackify.environment"),
            LabelKey::Service => write!(f, "{}", "local.stackify.service"),
            LabelKey::NodeVersion => write!(f, "{}", "local.stackify.node_version"),
            LabelKey::IsLeader => write!(f, "{}", "local.stackify.is_leader"),
        }
    }
}

impl Deref for LabelKey {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            LabelKey::Stackify => "local.stackify",
            LabelKey::EnvironmentName => "local.stackify.environment",
            LabelKey::Service => "local.stackify.service",
            LabelKey::NodeVersion => "local.stackify.node_version",
            LabelKey::IsLeader => "local.stackify.is_leader",
        }
    }
}

impl Into<String> for LabelKey {
    fn into(self) -> String {
        self.to_string()
    }
}

#[derive(Debug)]
pub struct StackifyLabel<T>(LabelKey, T)
where
    T: Into<String>;

impl<T> Into<(String, T)> for StackifyLabel<T>
where
    T: Into<String>,
{
    fn into(self) -> (String, T) {
        (self.0.to_string(), self.1)
    }
}

#[derive(Debug)]
pub struct DockerVersion {
    pub version: String,
    pub api_version: String,
    pub min_api_version: String,
    pub components: Vec<String>,
}

#[derive(Debug)]
pub struct StackifyNetwork {
    pub id: String,
    pub name: String,
}

#[derive(Debug)]
pub struct BuildStackifyBuildImage<'a> {
    pub user_id: u32,
    pub group_id: u32,
    pub bitcoin_version: String,
    pub pre_compile: bool,
    pub stackify_build_dockerfile: &'a [u8],
    pub stackify_cargo_config: &'a [u8],
}

pub struct StackifyContainer {
    pub id: String,
    pub name: String,
    pub labels: HashMap<String, String>,
    pub state: ContainerState,
    pub status_readable: String,
}

pub struct BuildStackifyRuntimeImage<'a> {
    pub user_id: u32,
    pub group_id: u32,
    pub stackify_runtime_dockerfile: &'a [u8],
}

pub struct BuildInfo {
    pub message: String,
    pub error: Option<String>,
    /// Progress tuple (current, total).
    pub progress: Option<Progress>,
}

pub struct LogEntry {
    pub message: String,
}

#[derive(Debug, Default)]
pub struct ListStackifyContainerOpts {
    pub environment_name: Option<EnvironmentName>,
    pub only_running: Option<bool>,
}

pub struct CreateContainerResult {
    pub id: String,
    pub warnings: Vec<String>,
}

pub struct Progress {
    pub current: u32,
    pub total: u32,
}

impl Progress {
    pub fn new(current: u32, total: u32) -> Self {
        Self { current, total }
    }

    pub fn percent(&self) -> u32 {
        self.current / self.total * 100
    }
}

pub struct StackifyImage {
    pub id: String,
    pub tags: Vec<String>,
    pub container_count: i64,
    pub size: i64,
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
