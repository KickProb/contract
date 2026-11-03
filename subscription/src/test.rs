#![cfg(test)]

extern crate std;

use super::{Error, KickProbSubscription, KickProbSubscriptionClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env,
};

fn create_usdc(env: &Env, admin: &Address) -> (Address, token::StellarAssetClient<'_>) {
    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    let sac_admin = token::StellarAssetClient::new(env, &sac.address());
    (sac.address(), sac_admin)
}

#[test]
fn initialize_and_set_plan() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    let (usdc, _) = create_usdc(&env, &admin);

    let contract_id = env.register(KickProbSubscription, ());
    let client = KickProbSubscriptionClient::new(&env, &contract_id);

    client.initialize(&admin, &usdc, &treasury);
    client.set_plan(&1_u32, &5_000_000_i128, &(30 * 24 * 60 * 60));

    let plan = client.plan(&1_u32).unwrap();
    assert_eq!(plan.price, 5_000_000);
    assert_eq!(plan.duration_seconds, 30 * 24 * 60 * 60);
}

#[test]
fn pay_extends_entitlement() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    let user = Address::generate(&env);
    let (usdc, usdc_admin) = create_usdc(&env, &admin);

    // Fund the user with two months of USDC.
    usdc_admin.mint(&user, &10_000_000_i128);

    let contract_id = env.register(KickProbSubscription, ());
    let client = KickProbSubscriptionClient::new(&env, &contract_id);
    client.initialize(&admin, &usdc, &treasury);
    client.set_plan(&1_u32, &5_000_000_i128, &(30 * 24 * 60 * 60));

    env.ledger().with_mut(|l| l.timestamp = 1_700_000_000);
    let ent1 = client.pay(&user, &1_u32);
    assert_eq!(ent1.plan_id, 1);
    assert_eq!(ent1.expires_at, 1_700_000_000 + 30 * 24 * 60 * 60);

    // Paying again before expiry stacks the duration.
    env.ledger().with_mut(|l| l.timestamp = 1_700_000_100);
    let ent2 = client.pay(&user, &1_u32);
    assert_eq!(ent2.expires_at, ent1.expires_at + 30 * 24 * 60 * 60);
}

#[test]
fn pay_unknown_plan_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    let user = Address::generate(&env);
    let (usdc, _) = create_usdc(&env, &admin);

    let contract_id = env.register(KickProbSubscription, ());
    let client = KickProbSubscriptionClient::new(&env, &contract_id);
    client.initialize(&admin, &usdc, &treasury);

    let err = client.try_pay(&user, &999_u32).err().unwrap().unwrap();
    assert_eq!(err, Error::PlanNotFound);
}
