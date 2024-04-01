pub mod docker;

pub mod api;
pub mod download;
#[cfg(test)]
pub mod tests;
pub mod util;

use std::fmt::Display;

use color_eyre::eyre::{bail, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServiceType {
    BitcoinMiner = 0,
    BitcoinFollower = 1,
    StacksMiner = 2,
    StacksFollower = 3,
    StacksSigner = 4,
    StacksStackerSelf = 5,
    StacksStackerPool = 6,
    StackifyEnvironment = 7,
    StackifyDaemon = 8,
}

impl Into<i32> for ServiceType {
    fn into(self) -> i32 {
        self as i32
    }
}

impl From<i32> for ServiceType {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::BitcoinMiner,
            1 => Self::BitcoinFollower,
            2 => Self::StacksMiner,
            3 => Self::StacksFollower,
            4 => Self::StacksSigner,
            5 => Self::StacksStackerSelf,
            6 => Self::StacksStackerPool,
            7 => Self::StackifyEnvironment,
            8 => Self::StackifyDaemon,
            _ => panic!("Invalid service type value: {}", value),
        }
    }
}

impl ServiceType {
    pub fn from_i32(value: i32) -> Result<Self> {
        match value {
            0 => Ok(Self::BitcoinMiner),
            1 => Ok(Self::BitcoinFollower),
            2 => Ok(Self::StacksMiner),
            3 => Ok(Self::StacksFollower),
            4 => Ok(Self::StacksSigner),
            5 => Ok(Self::StacksStackerSelf),
            6 => Ok(Self::StacksStackerPool),
            7 => Ok(Self::StackifyEnvironment),
            8 => Ok(Self::StackifyDaemon),
            _ => bail!("Invalid service type value: {}", value),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ServiceState {
    Running = 1,
    Stopped = 2,
    Error = 3,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ServiceAction {
    StartContainer = 1,
    StopContainer = 2,
    UpgradeService = 3,
    StartService = 4,
    StopService = 5,
    AttachNetwork = 6,
    DetachNetwork = 7,
}

pub enum FileType {
    Binary = 0,
    PlainText = 1,
    HandlebarsTemplate = 2,
}
