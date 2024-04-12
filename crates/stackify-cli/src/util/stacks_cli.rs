use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MakeKeychainResult {
    pub mnemonic: String,
    #[serde(rename = "keyInfo")]
    pub key_info: KeyInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyInfo {
    #[serde(rename = "privateKey")]
    pub private_key: String,
    #[serde(rename = "publicKey")]
    pub public_key: String,
    pub address: String,
    #[serde(rename = "btcAddress")]
    pub btc_address: String,
    pub wif: String,
    pub index: u32,
}

impl MakeKeychainResult {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}
