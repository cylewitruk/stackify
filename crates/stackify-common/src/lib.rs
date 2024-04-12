pub mod api;
pub mod download;
#[cfg(test)]
pub mod tests;
pub mod types;
pub mod util;

use color_eyre::eyre::{bail, Result};
use serde::{Deserialize, Serialize};

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

    pub fn is(&self, other: i32) -> bool {
        self.clone() as i32 == other
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FileType {
    Binary = 0,
    PlainText = 1,
    HandlebarsTemplate = 2,
}

impl FileType {
    pub fn from_i32(value: i32) -> Result<Self> {
        match value {
            0 => Ok(Self::Binary),
            1 => Ok(Self::PlainText),
            2 => Ok(Self::HandlebarsTemplate),
            _ => bail!("Invalid file type value: {}", value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValueType {
    String = 0,
    Integer = 1,
    Boolean = 2,
    Enum = 3,
    StacksKeychain = 4,
}

impl ValueType {
    pub fn from_i32(value: i32) -> Result<Self> {
        match value {
            0 => Ok(Self::String),
            1 => Ok(Self::Integer),
            2 => Ok(Self::Boolean),
            3 => Ok(Self::Enum),
            _ => bail!("Invalid value type value: {}", value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConfigElementKind {
    File,
    Param,
}
