use api::{hash::Hash160, transactions::AddressHashMode};
use libsecp256k1::PublicKey;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use sha2::{Digest, Sha256};

use crate::bitcoin::{opcodes, script::Script};

pub mod api;

pub const C32_ADDRESS_VERSION_MAINNET_SINGLESIG: u8 = 22; // P
pub const C32_ADDRESS_VERSION_MAINNET_MULTISIG: u8 = 20; // M
pub const C32_ADDRESS_VERSION_TESTNET_SINGLESIG: u8 = 26; // T
pub const C32_ADDRESS_VERSION_TESTNET_MULTISIG: u8 = 21; // N

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsensusHash(pub [u8; 20]);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StacksBlockId(pub [u8; 32]);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sha512Trunc256Sum(#[serde(with = "BigArray")] pub [u8; 32]);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockHeaderHash(#[serde(with = "BigArray")] pub [u8; 32]);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageSignature(#[serde(with = "BigArray")] pub [u8; 65]);

impl MessageSignature {
    pub fn empty() -> MessageSignature {
        // NOTE: this cannot be a valid signature
        MessageSignature([0u8; 65])
    }
}

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

impl StacksAddress {
    pub fn new(version: u8, hash: Hash160) -> StacksAddress {
        StacksAddress {
            version,
            bytes: hash,
        }
    }

    pub fn is_mainnet(&self) -> bool {
        match self.version {
            C32_ADDRESS_VERSION_MAINNET_MULTISIG | C32_ADDRESS_VERSION_MAINNET_SINGLESIG => true,
            C32_ADDRESS_VERSION_TESTNET_MULTISIG | C32_ADDRESS_VERSION_TESTNET_SINGLESIG => false,
            _ => false,
        }
    }

    pub fn burn_address(mainnet: bool) -> StacksAddress {
        StacksAddress {
            version: if mainnet {
                C32_ADDRESS_VERSION_MAINNET_SINGLESIG
            } else {
                C32_ADDRESS_VERSION_TESTNET_SINGLESIG
            },
            bytes: Hash160([0u8; 20]),
        }
    }

    /// Generate an address from a given address hash mode, signature threshold, and list of public
    /// keys.  Only return an address if the combination given is supported.
    /// The version is may be arbitrary.
    pub fn from_public_keys(
        version: u8,
        hash_mode: &AddressHashMode,
        num_sigs: usize,
        pubkeys: &Vec<PublicKey>,
    ) -> Option<StacksAddress> {
        // must be sufficient public keys
        if pubkeys.len() < num_sigs {
            return None;
        }

        // address hash mode must be consistent with the number of keys
        match *hash_mode {
            AddressHashMode::SerializeP2PKH | AddressHashMode::SerializeP2WPKH => {
                // must be a single public key, and must require one signature
                if num_sigs != 1 || pubkeys.len() != 1 {
                    return None;
                }
            }
            _ => {}
        }

        let hash_bits = public_keys_to_address_hash(hash_mode, num_sigs, pubkeys);
        Some(StacksAddress::new(version, hash_bits))
    }

    /// Make a P2PKH StacksAddress
    pub fn p2pkh(mainnet: bool, pubkey: &PublicKey) -> StacksAddress {
        let bytes = to_bits_p2pkh(pubkey);
        Self::p2pkh_from_hash(mainnet, bytes)
    }

    /// Make a P2PKH StacksAddress
    pub fn p2pkh_from_hash(mainnet: bool, hash: Hash160) -> StacksAddress {
        let version = if mainnet {
            C32_ADDRESS_VERSION_MAINNET_SINGLESIG
        } else {
            C32_ADDRESS_VERSION_TESTNET_SINGLESIG
        };
        Self {
            version,
            bytes: hash,
        }
    }
}

/// Convert a number of required signatures and a list of public keys into a byte-vec to hash to an
/// address.  Validity of the hash_flag vis a vis the num_sigs and pubkeys will _NOT_ be checked.
/// This is a low-level method.  Consider using StacksAdress::from_public_keys() if you can.
pub fn public_keys_to_address_hash(
    hash_flag: &AddressHashMode,
    num_sigs: usize,
    pubkeys: &Vec<PublicKey>,
) -> Hash160 {
    match *hash_flag {
        AddressHashMode::SerializeP2PKH => to_bits_p2pkh(&pubkeys[0]),
        AddressHashMode::SerializeP2SH => to_bits_p2sh(num_sigs, pubkeys),
        AddressHashMode::SerializeP2WPKH => to_bits_p2sh_p2wpkh(&pubkeys[0]),
        AddressHashMode::SerializeP2WSH => to_bits_p2sh_p2wsh(num_sigs, pubkeys),
    }
}

/// Internally, the Stacks blockchain encodes address the same as Bitcoin
/// single-sig address (p2pkh)
/// Get back the hash of the address
pub fn to_bits_p2pkh(pubk: &PublicKey) -> Hash160 {
    Hash160::from_data(&pubk.serialize_compressed())
}

struct BitcoinBuilder(Vec<u8>);

impl BitcoinBuilder {
    /// Creates a new empty script
    pub fn new() -> BitcoinBuilder {
        BitcoinBuilder(vec![])
    }

    /// The length in bytes of the script
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether the script is the empty script
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Adds instructions to push an integer onto the stack. Integers are
    /// encoded as little-endian signed-magnitude numbers, but there are
    /// dedicated opcodes to push some small integers.
    pub fn push_int(mut self, data: i64) -> BitcoinBuilder {
        // We can special-case -1, 1-16
        if data == -1 || (1..=16).contains(&data) {
            self.0.push((data - 1 + opcodes::OP_TRUE as i64) as u8);
            self
        }
        // We can also special-case zero
        else if data == 0 {
            self.0.push(opcodes::OP_FALSE as u8);
            self
        }
        // Otherwise encode it as data
        else {
            self.push_scriptint(data)
        }
    }

    /// Adds instructions to push an integer onto the stack, using the explicit
    /// encoding regardless of the availability of dedicated opcodes.
    pub fn push_scriptint(self, data: i64) -> BitcoinBuilder {
        self.push_slice(&crate::bitcoin::script::build_scriptint(data))
    }

    /// Adds instructions to push some arbitrary data onto the stack
    pub fn push_slice(mut self, data: &[u8]) -> BitcoinBuilder {
        // Start with a PUSH opcode
        match data.len() as u64 {
            n if n < opcodes::Ordinary::OP_PUSHDATA1 as u64 => {
                self.0.push(n as u8);
            }
            n if n < 0x100 => {
                self.0.push(opcodes::Ordinary::OP_PUSHDATA1 as u8);
                self.0.push(n as u8);
            }
            n if n < 0x10000 => {
                self.0.push(opcodes::Ordinary::OP_PUSHDATA2 as u8);
                self.0.push((n % 0x100) as u8);
                self.0.push((n / 0x100) as u8);
            }
            n if n < 0x100000000 => {
                self.0.push(opcodes::Ordinary::OP_PUSHDATA4 as u8);
                self.0.push((n % 0x100) as u8);
                self.0.push(((n / 0x100) % 0x100) as u8);
                self.0.push(((n / 0x10000) % 0x100) as u8);
                self.0.push((n / 0x1000000) as u8);
            }
            _ => panic!("tried to put a 4bn+ sized object into a script!"),
        }
        // Then push the acraw
        self.0.extend(data.iter().cloned());
        self
    }

    /// Adds a single opcode to the script
    pub fn push_opcode(mut self, data: opcodes::All) -> BitcoinBuilder {
        self.0.push(data as u8);
        self
    }

    /// Converts the `Builder` into an unmodifiable `Script`
    pub fn into_script(self) -> Script {
        Script(self.0.into_boxed_slice())
    }
}

/// Internally, the Stacks blockchain encodes address the same as Bitcoin
/// multi-sig address (p2sh)
fn to_bits_p2sh(num_sigs: usize, pubkeys: &Vec<PublicKey>) -> Hash160 {
    let mut bldr = BitcoinBuilder::new();
    bldr = bldr.push_int(num_sigs as i64);
    for pubk in pubkeys {
        bldr = bldr.push_slice(&pubk.serialize_compressed());
    }
    bldr = bldr.push_int(pubkeys.len() as i64);
    bldr = bldr.push_opcode(opcodes::All::OP_CHECKMULTISIG);

    let script = bldr.into_script();
    Hash160::from_data(&script.0)
}

/// Internally, the Stacks blockchain encodes address the same as Bitcoin
/// single-sig address over p2sh (p2h-p2wpkh)
fn to_bits_p2sh_p2wpkh(pubk: &PublicKey) -> Hash160 {
    let key_hash = Hash160::from_data(&pubk.serialize_compressed());

    let bldr = BitcoinBuilder::new().push_int(0).push_slice(&key_hash.0);

    let script = bldr.into_script();
    Hash160::from_data(&script.0)
}

/// Internally, the Stacks blockchain encodes address the same as Bitcoin
/// multisig address over p2sh (p2sh-p2wsh)
fn to_bits_p2sh_p2wsh(num_sigs: usize, pubkeys: &Vec<PublicKey>) -> Hash160 {
    let mut bldr = BitcoinBuilder::new();
    bldr = bldr.push_int(num_sigs as i64);
    for pubk in pubkeys {
        bldr = bldr.push_slice(&pubk.serialize_compressed());
    }
    bldr = bldr.push_int(pubkeys.len() as i64);
    bldr = bldr.push_opcode(opcodes::All::OP_CHECKMULTISIG);

    let mut digest = Sha256::new();
    let mut d = [0u8; 32];

    digest.update(&bldr.into_script().0);
    d.copy_from_slice(digest.finalize().as_slice());

    let ws = BitcoinBuilder::new()
        .push_int(0)
        .push_slice(&d)
        .into_script();
    Hash160::from_data(&ws.0)
}
