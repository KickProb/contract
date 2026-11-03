#![cfg(test)]

extern crate std;

use super::{Error, KickProbReceipt, KickProbReceiptClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, BytesN, Env,
};

fn bytes32(env: &Env, seed: u8) -> BytesN<32> {
    BytesN::from_array(env, &[seed; 32])
}

#[test]
fn publish_then_settle_happy_path() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let operator = Address::generate(&env);

    let contract_id = env.register(KickProbReceipt, ());
    let client = KickProbReceiptClient::new(&env, &contract_id);
    client.initialize(&admin, &operator);

    let batch = bytes32(&env, 0xAB);
    let model = bytes32(&env, 0xCD);

    env.ledger().with_mut(|l| l.timestamp = 1_700_000_000);
    let published = client.publish_receipt(&batch, &model, &1_700_000_100_u64, &1_700_200_000_u64);
    assert_eq!(published.window_start, 1_700_000_100);
    assert!(published.settled_hash.is_none());

    env.ledger().with_mut(|l| l.timestamp = 1_700_400_000);
    let outcome = bytes32(&env, 0xEF);
    let settled = client.settle_receipt(&batch, &outcome);
    assert_eq!(settled.settled_hash.unwrap(), outcome);
    assert_eq!(settled.settled_at.unwrap(), 1_700_400_000);
}

#[test]
fn publish_twice_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let operator = Address::generate(&env);

    let contract_id = env.register(KickProbReceipt, ());
    let client = KickProbReceiptClient::new(&env, &contract_id);
    client.initialize(&admin, &operator);

    let batch = bytes32(&env, 0x01);
    let model = bytes32(&env, 0x02);

    client.publish_receipt(&batch, &model, &1_u64, &2_u64);
    let err = client
        .try_publish_receipt(&batch, &model, &1_u64, &2_u64)
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(err, Error::AlreadyPublished);
}
