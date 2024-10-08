use std::{fmt::Display, ops::Deref, path::PathBuf};

use color_eyre::{eyre::bail, Result};
use regex::Regex;
use serde::Serialize;

use crate::{FileType, ServiceType, ValueType};

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EnvironmentName {
    // [a-z0-9]+(?:[._-]{1,2}[a-z0-9]+)*
    name: String,
}

impl EnvironmentName {
    pub fn new(name: &str) -> Result<Self> {
        // This is the Regex used by Docker for names.
        let regex = Regex::new("[a-z0-9]+(?:[._-]{1,2}[a-z0-9]+)*")?;
        if !regex.is_match(name) {
            bail!(format!("The environment name '{}' is invalid.", name));
        }

        Ok(Self {
            name: name.to_string(),
        })
    }

    pub fn as_str(&self) -> &str {
        &self.name
    }
}

impl Display for EnvironmentName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Deref for EnvironmentName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.name
    }
}

impl AsRef<str> for EnvironmentName {
    fn as_ref(&self) -> &str {
        &self.name
    }
}

impl Into<String> for EnvironmentName {
    fn into(self) -> String {
        self.name
    }
}

impl Into<String> for &EnvironmentName {
    fn into(self) -> String {
        self.name.clone()
    }
}

impl TryFrom<&str> for EnvironmentName {
    type Error = color_eyre::eyre::Error;

    fn try_from(name: &str) -> Result<Self> {
        Self::new(name)
    }
}

impl TryFrom<String> for EnvironmentName {
    type Error = color_eyre::eyre::Error;

    fn try_from(name: String) -> Result<Self> {
        Self::new(&name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortMap {
    pub host_port: u16,
    pub container_port: u16,
    pub protocol: NetworkProtocol,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkProtocol {
    Tcp = 1,
    Udp = 2,
    Sctp = 3,
}

impl TryFrom<i32> for NetworkProtocol {
    type Error = color_eyre::eyre::Error;

    fn try_from(value: i32) -> Result<Self> {
        match value {
            1 => Ok(NetworkProtocol::Tcp),
            2 => Ok(NetworkProtocol::Udp),
            3 => Ok(NetworkProtocol::Sctp),
            _ => bail!("Invalid network protocol value: {}", value),
        }
    }
}   

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentService {
    pub id: i32,
    pub service_type: ServiceTypeSimple,
    pub version: ServiceVersion,
    pub name: String,
    pub remark: Option<String>,
    pub file_headers: Vec<EnvironmentServiceFileHeader>,
    pub params: Vec<EnvironmentServiceParam>,
    pub port_mappings: Vec<PortMap>,
}

impl EnvironmentService {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentEpoch {
    pub id: i32,
    pub epoch: Epoch,
    pub starts_at_block_height: u32,
    pub ends_at_block_height: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceTypeSimple {
    pub id: i32,
    pub name: String,
    pub cli_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceTypePort {
    pub id: i32,
    pub service_type: ServiceType,
    pub port: u16,
    pub protocol: NetworkProtocol,
    pub remark: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceTypeFull {
    pub id: i32,
    pub name: String,
    pub cli_name: String,
    pub allow_max_epoch: bool,
    pub allow_min_epoch: bool,
    pub allow_git_target: bool,
    pub versions: Vec<ServiceVersion>,
    pub ports: Vec<ServiceTypePort>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceVersion {
    pub id: i32,
    pub version: String,
    pub min_epoch: Option<Epoch>,
    pub max_epoch: Option<Epoch>,
    pub git_target: Option<GitTarget>,
    pub cli_name: String,
    pub rebuild_required: bool,
    pub last_built_at: Option<time::PrimitiveDateTime>,
    pub last_build_commit_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Epoch {
    pub id: i32,
    pub name: String,
    pub default_block_height: u32,
}

impl Display for Epoch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.default_block_height)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Environment {
    pub id: i32,
    pub name: EnvironmentName,
    pub services: Vec<EnvironmentService>,
    pub epochs: Vec<EnvironmentEpoch>,
    pub keychains: Vec<EnvironmentKeychain>,
}

impl Environment {
    pub fn find_service_instances(
        &self,
        service_type: ServiceType,
        name: &str,
    ) -> Vec<&EnvironmentService> {
        let service_type_id = service_type as i32;

        self.services
            .iter()
            .filter(|service| service.service_type.id == service_type_id)
            .filter(|svc| &svc.name != name)
            .collect::<Vec<_>>()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitTargetKind {
    Tag,
    Branch,
    Commit,
}

impl Display for GitTargetKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitTargetKind::Tag => write!(f, "tag"),
            GitTargetKind::Branch => write!(f, "branch"),
            GitTargetKind::Commit => write!(f, "commit"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitTarget {
    pub target_type: GitTargetKind,
    pub target: String,
}

impl GitTarget {
    pub fn new(target_type: GitTargetKind, target: &str) -> Self {
        Self {
            target_type,
            target: target.to_string(),
        }
    }

    pub fn parse<T: AsRef<str>>(s: T) -> Option<GitTarget> {
        let s = s.as_ref();
        let split = s.split(":").collect::<Vec<_>>();
        if split.len() < 2 {
            return None;
        }
        let target_type = match split[0] {
            "tag" => GitTargetKind::Tag,
            "branch" => GitTargetKind::Branch,
            "commit" => GitTargetKind::Commit,
            _ => return None,
        };
        Some(GitTarget {
            target_type,
            target: split[1].to_string(),
        })
    }

    pub fn parse_opt<T: AsRef<str>>(s: Option<T>) -> Option<GitTarget> {
        if let Some(s) = s {
            Self::parse(s)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct ServiceTypeFileHeader {
    pub id: i32,
    pub service_type: ServiceTypeSimple,
    pub file_type: FileType,
    pub filename: String,
    pub destination_dir: PathBuf,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentServiceFileHeader {
    pub id: i32,
    pub service_type: ServiceType,
    pub file_type: FileType,
    pub filename: String,
    pub destination_dir: PathBuf,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct ServiceFileContents {
    pub contents: Vec<u8>,
}

pub struct EnvironmentServiceFile {
    pub header: EnvironmentServiceFileHeader,
    pub contents: ServiceFileContents,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceTypeParam {
    pub id: i32,
    pub service_type: ServiceTypeSimple,
    pub name: String,
    pub key: String,
    pub description: String,
    pub default_value: String,
    pub is_required: bool,
    pub value_type: ValueType,
    pub allowed_values: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentServiceParam {
    pub id: i32,
    pub param: ServiceTypeParam,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EnvironmentKeychain {
    pub id: i32,
    pub environment_id: i32,
    pub stx_address: String,
    pub amount: u64,
    pub mnemonic: String,
    pub private_key: String,
    pub public_key: String,
    pub btc_address: String,
    pub remark: Option<String>,
}
