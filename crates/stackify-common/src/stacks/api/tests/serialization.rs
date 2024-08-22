use std::u32;

use libsecp256k1::{PublicKey, SecretKey};
use rand::thread_rng;

use crate::stacks::{
    api::{
        clarity::ClarityVersion,
        transactions::{
            SinglesigHashMode, SinglesigSpendingCondition, StacksTransaction,
            TransactionAnchorMode, TransactionAuth, TransactionPayload,
            TransactionPostConditionMode, TransactionSmartContract, TransactionSpendingCondition,
            TransactionVersion,
        },
    },
    MessageSignature, TransactionPublicKeyEncoding,
};

#[test]
pub fn test_publish_contract() {
    let src = r#"(define-public (hello-world) (ok "Hello, world!"))"#;
    let payload = TransactionPayload::SmartContract(
        TransactionSmartContract {
            name: "hello-world".to_string(),
            code_body: src.to_string(),
        },
        Some(ClarityVersion::Clarity2),
    );
    let private_key = SecretKey::random(&mut thread_rng());
    let public_key = PublicKey::from_secret_key(&private_key);

    let trx = StacksTransaction {
        version: TransactionVersion::Testnet,
        chain_id: u32::MAX,
        anchor_mode: TransactionAnchorMode::OnChainOnly,
        auth: TransactionAuth::Standard(TransactionSpendingCondition::Singlesig(
            SinglesigSpendingCondition {
                hash_mode: SinglesigHashMode::P2PKH,
                key_encoding: TransactionPublicKeyEncoding::Compressed,
                nonce: 45,
                tx_fee: 1234,
                signature: MessageSignature::empty(),
                signer: crate::stacks::api::hash::Hash160::from_public_key(&public_key),
            },
        )),
        post_condition_mode: TransactionPostConditionMode::Deny,
        post_conditions: vec![],
        payload,
    };

    let trx_json = serde_json::to_string(&trx).unwrap();
}
