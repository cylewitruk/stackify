use api::hash20::Hash160;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

pub mod api;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsensusHash(pub [u8; 20]);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StacksBlockId(pub [u8; 32]);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sha512Trunc256Sum(
    #[serde(with = "BigArray")]
    pub [u8; 32],
);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockHeaderHash(
    #[serde(with = "BigArray")]
    pub [u8; 32]
);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageSignature(
    #[serde(with = "BigArray")]
    pub [u8; 65],
);

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize)]
pub enum TransactionPublicKeyEncoding {
    // ways we can encode a public key
    Compressed = 0x00,
    Uncompressed = 0x01,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct StacksAddress {
    pub version: u8,
    pub bytes: Hash160,
}