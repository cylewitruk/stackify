use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid Crockford32 string")]
    InvalidCrockford32,
    #[error("Invalid version: {0}")]
    InvalidVersion(u8),
    #[error("Data is empty")]
    EmptyData,
    /// Invalid character encountered
    #[error("Invalid character: {0}")]
    BadByte(u8),
    /// Checksum was not correct (expected, actual)
    #[error("Bad checksum: expected {0}, got {1}")]
    BadChecksum(u32, u32),
    /// The length (in bytes) of the object was not correct
    /// Note that if the length is excessively long the provided length may be
    /// an estimate (and the checksum step may be skipped).
    #[error("Invalid length: {0}")]
    InvalidLength(usize),
    /// Checked data was less than 4 bytes
    #[error("Data too short: {0}")]
    TooShort(usize),
    /// Any other error
    #[error("Error: {0}")]
    Other(String),
}
