#![no_std]

mod allowlist;
pub use allowlist::DataKey;

use soroban_sdk::{contract, contracterror, contractimpl, Address, Env, Symbol};

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    Unauthorized = 1,
    ContractPaused = 2,
}

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

    pub fn allow_address(env: Env, admin: Address, address: Address) -> Result<(), ContractError> {
        Self::require_admin(&env, &admin)?;
        Self::require_not_paused(&env)?;
        env.storage()
            .persistent()
            .set(&DataKey::Allowed(address.clone()), &true);
        env.events()
            .publish((Symbol::new(&env, "address_allowed"),), address);
        Ok(())
    }

    // Emergency policy: block_address and clear_address are permitted while paused
    // so the admin can remediate compromised addresses without unpausing first.
    pub fn block_address(env: Env, admin: Address, address: Address) -> Result<(), ContractError> {
        Self::require_admin(&env, &admin)?;
        env.storage()
            .persistent()
            .set(&DataKey::Blocked(address.clone()), &true);
        env.events()
            .publish((Symbol::new(&env, "address_blocked"),), address);
        Ok(())
    }

    pub fn clear_address(env: Env, admin: Address, address: Address) -> Result<(), ContractError> {
        Self::require_admin(&env, &admin)?;
        env.storage()
            .persistent()
            .set(&DataKey::Blocked(address.clone()), &false);
        env.storage()
            .persistent()
            .set(&DataKey::Allowed(address.clone()), &true);
        env.events()
            .publish((Symbol::new(&env, "address_cleared"),), address);
        Ok(())
    }

    pub fn pause(env: Env, admin: Address) -> Result<(), ContractError> {
        Self::require_admin(&env, &admin)?;
        env.storage().instance().set(&DataKey::Paused, &true);
        env.events()
            .publish((Symbol::new(&env, "compliance_paused"),), admin);
        Ok(())
    }

    pub fn unpause(env: Env, admin: Address) -> Result<(), ContractError> {
        Self::require_admin(&env, &admin)?;
        env.storage().instance().set(&DataKey::Paused, &false);
        env.events()
            .publish((Symbol::new(&env, "compliance_unpaused"),), admin);
        Ok(())
    }

    fn require_admin(env: &Env, admin: &Address) -> Result<(), ContractError> {
        admin.require_auth();
        let stored: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if stored != *admin {
            return Err(ContractError::Unauthorized);
        }
        Ok(())
    }

    fn require_not_paused(env: &Env) -> Result<(), ContractError> {
        let paused: bool = env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false);
        if paused {
            return Err(ContractError::ContractPaused);
        }
        Ok(())
    }
}

#[cfg(test)]
extern crate std;
