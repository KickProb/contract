#![no_std]

//! KickProb prediction-receipt contract.
//!
//! `publish_receipt` is called by the KickProb publisher at kickoff-
//! freeze time for every batch of predictions. It commits:
//!
//! * `batch_hash`     — SHA-256 of the canonical JSON of the batch
//! * `model_version`  — e.g. b"v24.11-champion"
//! * `window_start`   — earliest kickoff UNIX ts in the batch
//! * `window_end`     — latest kickoff UNIX ts in the batch
//!
//! Once a match window is fully played, `settle_receipt` records the
//! aggregate outcome hash (wins / losses / pushes) for calibration.
//!
//! Only the configured operator (set once by admin) may publish or
//! settle. Everything else is a read-only lookup for indexers and the
//! public KickProb site.

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, Address, BytesN, Env,
};

#[contracttype]
#[derive(Clone)]
pub struct Receipt {
    pub batch_hash: BytesN<32>,
    pub model_version: BytesN<32>,
    pub window_start: u64,
    pub window_end: u64,
    pub published_at: u64,
    pub settled_hash: Option<BytesN<32>>,
    pub settled_at: Option<u64>,
}

#[contracttype]
pub enum DataKey {
    Admin,
    Operator,
    Receipt(BytesN<32>),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    AlreadyPublished = 3,
    NotFound = 4,
    AlreadySettled = 5,
}

#[contractevent]
pub struct Published {
    #[topic]
    pub batch_hash: BytesN<32>,
    #[topic]
    pub model_version: BytesN<32>,
    pub window_start: u64,
    pub window_end: u64,
    pub published_at: u64,
}

#[contractevent]
pub struct Settled {
    #[topic]
    pub batch_hash: BytesN<32>,
    pub outcome_hash: BytesN<32>,
    pub settled_at: u64,
}

#[contract]
pub struct KickProbReceipt;

#[contractimpl]
impl KickProbReceipt {
    pub fn initialize(env: Env, admin: Address, operator: Address) -> Result<(), Error> {
        let storage = env.storage().instance();
        if storage.has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        storage.set(&DataKey::Admin, &admin);
        storage.set(&DataKey::Operator, &operator);
        Ok(())
    }

    /// Admin rotates the operator key used by the backend publisher.
    pub fn set_operator(env: Env, operator: Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        admin.require_auth();
        env.storage().instance().set(&DataKey::Operator, &operator);
        Ok(())
    }

    pub fn publish_receipt(
        env: Env,
        batch_hash: BytesN<32>,
        model_version: BytesN<32>,
        window_start: u64,
        window_end: u64,
    ) -> Result<Receipt, Error> {
        Self::require_operator(&env)?;

        let key = DataKey::Receipt(batch_hash.clone());
        if env.storage().persistent().has(&key) {
            return Err(Error::AlreadyPublished);
        }

        let now = env.ledger().timestamp();
        let receipt = Receipt {
            batch_hash: batch_hash.clone(),
            model_version: model_version.clone(),
            window_start,
            window_end,
            published_at: now,
            settled_hash: None,
            settled_at: None,
        };
        env.storage().persistent().set(&key, &receipt);

        Published {
            batch_hash,
            model_version,
            window_start,
            window_end,
            published_at: now,
        }
        .publish(&env);

        Ok(receipt)
    }

    pub fn settle_receipt(
        env: Env,
        batch_hash: BytesN<32>,
        outcome_hash: BytesN<32>,
    ) -> Result<Receipt, Error> {
        Self::require_operator(&env)?;

        let key = DataKey::Receipt(batch_hash.clone());
        let mut receipt: Receipt = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::NotFound)?;

        if receipt.settled_hash.is_some() {
            return Err(Error::AlreadySettled);
        }
        let now = env.ledger().timestamp();
        receipt.settled_hash = Some(outcome_hash.clone());
        receipt.settled_at = Some(now);
        env.storage().persistent().set(&key, &receipt);

        Settled {
            batch_hash,
            outcome_hash,
            settled_at: now,
        }
        .publish(&env);

        Ok(receipt)
    }

    pub fn receipt_of(env: Env, batch_hash: BytesN<32>) -> Option<Receipt> {
        env.storage()
            .persistent()
            .get(&DataKey::Receipt(batch_hash))
    }

    pub fn operator(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Operator)
    }

    pub fn admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }

    fn require_operator(env: &Env) -> Result<(), Error> {
        let operator: Address = env
            .storage()
            .instance()
            .get(&DataKey::Operator)
            .ok_or(Error::NotInitialized)?;
        operator.require_auth();
        Ok(())
    }
}

#[cfg(test)]
mod test;
