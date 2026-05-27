#![no_std]

mod allowlist;
pub use allowlist::DataKey;

use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};

#[contract]
pub struct ComplianceContract;

#[contractimpl]
impl ComplianceContract {
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
    }

    pub fn is_allowed(env: Env, address: Address) -> bool {
        let blocked: bool = env
            .storage()
            .persistent()
            .get(&DataKey::Blocked(address.clone()))
            .unwrap_or(false);
        let allowed: bool = env
            .storage()
            .persistent()
            .get(&DataKey::Allowed(address))
            .unwrap_or(false);
        allowed && !blocked
    }

    pub fn allow_address(env: Env, admin: Address, address: Address) {
        Self::require_admin(&env, &admin);
        Self::require_not_paused(&env);
        env.storage()
            .persistent()
            .set(&DataKey::Allowed(address.clone()), &true);
        env.events()
            .publish((Symbol::new(&env, "address_allowed"),), address);
    }

    // Emergency policy: block_address and clear_address are permitted while paused
    // so the admin can remediate compromised addresses without unpausing first.
    pub fn block_address(env: Env, admin: Address, address: Address) {
        Self::require_admin(&env, &admin);
        env.storage()
            .persistent()
            .set(&DataKey::Blocked(address.clone()), &true);
        env.events()
            .publish((Symbol::new(&env, "address_blocked"),), address);
    }

    pub fn clear_address(env: Env, admin: Address, address: Address) {
        Self::require_admin(&env, &admin);
        env.storage()
            .persistent()
            .set(&DataKey::Blocked(address.clone()), &false);
        env.storage()
            .persistent()
            .set(&DataKey::Allowed(address.clone()), &true);
        env.events()
            .publish((Symbol::new(&env, "address_cleared"),), address);
    }

    pub fn pause(env: Env, admin: Address) {
        Self::require_admin(&env, &admin);
        env.storage().instance().set(&DataKey::Paused, &true);
        env.events()
            .publish((Symbol::new(&env, "compliance_paused"),), admin);
    }

    pub fn unpause(env: Env, admin: Address) {
        Self::require_admin(&env, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.events()
            .publish((Symbol::new(&env, "compliance_unpaused"),), admin);
    }

    fn require_admin(env: &Env, admin: &Address) {
        admin.require_auth();
        let stored: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if stored != *admin {
            panic!("Unauthorized");
        }
    }

    fn require_not_paused(env: &Env) {
        let paused: bool = env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false);
        if paused {
            panic!("ContractPaused");
        }
    }
}

#[cfg(test)]
extern crate std;
