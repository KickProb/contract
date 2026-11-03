#![no_std]

//! KickProb subscription contract.
//!
//! Users pay for a KickProb plan in USDC (via the Stellar Asset Contract)
//! and this contract records the resulting entitlement. The KickProb
//! backend later calls `entitlement_of` to decide whether a user may see
//! premium predictions.
//!
//! Plans are identified by a `u32` plan id. The admin configures each
//! plan's price (in USDC stroops) and duration (in ledger seconds).

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, token, Address, Env,
    Symbol,
};

#[contracttype]
#[derive(Clone)]
pub struct Plan {
    pub price: i128,
    pub duration_seconds: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct Entitlement {
    pub plan_id: u32,
    pub expires_at: u64,
}

#[contracttype]
pub enum DataKey {
    Admin,
    Usdc,
    Treasury,
    Plan(u32),
    Entitlement(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    PlanNotFound = 3,
    Unauthorized = 4,
}

#[contractevent]
pub struct Paid {
    #[topic]
    pub user: Address,
    #[topic]
    pub plan_id: u32,
    pub amount: i128,
    pub expires_at: u64,
}

#[contractevent]
pub struct PlanSet {
    #[topic]
    pub plan_id: u32,
    pub price: i128,
    pub duration_seconds: u64,
}

#[contract]
pub struct KickProbSubscription;

#[contractimpl]
impl KickProbSubscription {
    /// One-shot bootstrap. `usdc` is the Stellar Asset Contract address
    /// for USDC on the target network; `treasury` receives subscription
    /// payments.
    pub fn initialize(
        env: Env,
        admin: Address,
        usdc: Address,
        treasury: Address,
    ) -> Result<(), Error> {
        let storage = env.storage().instance();
        if storage.has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        storage.set(&DataKey::Admin, &admin);
        storage.set(&DataKey::Usdc, &usdc);
        storage.set(&DataKey::Treasury, &treasury);
        Ok(())
    }

    /// Admin-only. Register or update a plan.
    pub fn set_plan(
        env: Env,
        plan_id: u32,
        price: i128,
        duration_seconds: u64,
    ) -> Result<(), Error> {
        Self::require_admin(&env)?;
        let plan = Plan {
            price,
            duration_seconds,
        };
        env.storage().persistent().set(&DataKey::Plan(plan_id), &plan);
        PlanSet {
            plan_id,
            price,
            duration_seconds,
        }
        .publish(&env);
        Ok(())
    }

    /// Pay for a plan. Requires the caller to authorize the USDC
    /// transfer. On success, extends or creates the user's entitlement
    /// and emits `Paid`.
    pub fn pay(env: Env, user: Address, plan_id: u32) -> Result<Entitlement, Error> {
        user.require_auth();

        let plan: Plan = env
            .storage()
            .persistent()
            .get(&DataKey::Plan(plan_id))
            .ok_or(Error::PlanNotFound)?;

        let usdc: Address = env
            .storage()
            .instance()
            .get(&DataKey::Usdc)
            .ok_or(Error::NotInitialized)?;
        let treasury: Address = env
            .storage()
            .instance()
            .get(&DataKey::Treasury)
            .ok_or(Error::NotInitialized)?;

        token::Client::new(&env, &usdc).transfer(&user, &treasury, &plan.price);

        let now = env.ledger().timestamp();
        let existing: Option<Entitlement> = env
            .storage()
            .persistent()
            .get(&DataKey::Entitlement(user.clone()));
        let base = existing
            .map(|e| if e.expires_at > now { e.expires_at } else { now })
            .unwrap_or(now);
        let expires_at = base.saturating_add(plan.duration_seconds);

        let entitlement = Entitlement {
            plan_id,
            expires_at,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Entitlement(user.clone()), &entitlement);

        Paid {
            user,
            plan_id,
            amount: plan.price,
            expires_at,
        }
        .publish(&env);

        Ok(entitlement)
    }

    pub fn entitlement_of(env: Env, user: Address) -> Option<Entitlement> {
        env.storage()
            .persistent()
            .get(&DataKey::Entitlement(user))
    }

    pub fn plan(env: Env, plan_id: u32) -> Option<Plan> {
        env.storage().persistent().get(&DataKey::Plan(plan_id))
    }

    pub fn admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }

    fn require_admin(env: &Env) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        admin.require_auth();
        Ok(())
    }
}

// Avoid "unused" warnings on Symbol in minimal builds.
#[allow(dead_code)]
const _TAG: Symbol = soroban_sdk::symbol_short!("kickprob");
