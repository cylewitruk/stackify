use std::fmt::Display;

use color_eyre::{eyre::bail, Result};
use regex::Regex;

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
pub struct EnvironmentService {
    pub id: i32,
    pub service_type: ServiceTypeSimple,
    pub version: ServiceVersion,
    pub name: String,
    pub remark: Option<String>,
}

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
pub struct ServiceTypeFull {
    pub id: i32,
    pub name: String,
    pub cli_name: String,
    pub allow_max_epoch: bool,
    pub allow_min_epoch: bool,
    pub allow_git_target: bool,
    pub versions: Vec<ServiceVersion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceVersion {
    pub id: i32,
    pub version: String,
    pub min_epoch: Option<Epoch>,
    pub max_epoch: Option<Epoch>,
    pub git_target: Option<GitTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Epoch {
    pub id: i32,
    pub name: String,
    pub default_block_height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Environment {
    pub name: EnvironmentName,
    pub services: Vec<EnvironmentService>,
    pub epochs: Vec<EnvironmentEpoch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitTargetKind {
    Tag,
    Branch,
    Commit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitTarget {
    pub target_type: GitTargetKind,
    pub target: String,
}

impl GitTarget {
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