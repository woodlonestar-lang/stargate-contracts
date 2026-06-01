#![no_std]

mod multisig;
mod settlement;

pub use multisig::{DataKey, Settlement, SettlementStatus};

use settlement::{approval_weight, require_authorized_signer};
use soroban_sdk::{contract, contractimpl, token, Address, Env, Symbol, Vec};

#[contract]
pub struct TreasuryContract;

#[contractimpl]
impl TreasuryContract {
    pub fn initialize(env: Env, admin: Address, threshold: u32) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::Threshold, &threshold);
        env.storage()
            .instance()
            .set(&DataKey::SettlementCount, &0u64);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage()
            .instance()
            .set(&DataKey::Signer(admin.clone()), &1u32);
        env.events()
            .publish((Symbol::new(&env, "treasury_initialized"),), admin);
    }

    pub fn set_signer(env: Env, admin: Address, signer: Address, weight: u32) {
        Self::require_admin(&env, &admin);
        env.storage()
            .instance()
            .set(&DataKey::Signer(signer.clone()), &weight);
        env.events()
            .publish((Symbol::new(&env, "signer_weight_set"), signer), weight);
    }

    pub fn propose_settlement(
        env: Env,
        signer: Address,
        merchant_address: Address,
        amount: i128,
    ) -> u64 {
        Self::require_not_paused(&env);
        require_authorized_signer(&env, &signer);
        if amount <= 0 {
            panic!("InvalidAmount");
        }

        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::SettlementCount)
            .unwrap_or(0);
        let id = count + 1;
        let mut approvals = Vec::new(&env);
        approvals.push_back(signer);
        let settlement = Settlement {
            id,
            merchant_address,
            amount,
            approvals,
            status: SettlementStatus::Pending,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Settlement(id), &settlement);
        env.storage().instance().set(&DataKey::SettlementCount, &id);
        env.events()
            .publish((Symbol::new(&env, "settlement_proposed"), id), settlement);
        id
    }

    pub fn approve_settlement(env: Env, signer: Address, settlement_id: u64) -> Settlement {
        Self::require_not_paused(&env);
        require_authorized_signer(&env, &signer);
        let mut settlement: Settlement = env
            .storage()
            .persistent()
            .get(&DataKey::Settlement(settlement_id))
            .unwrap();
        if settlement.status != SettlementStatus::Pending {
            panic!("AlreadyExecuted");
        }
        if !settlement.approvals.contains(&signer) {
            settlement.approvals.push_back(signer);
        }
        env.storage()
            .persistent()
            .set(&DataKey::Settlement(settlement_id), &settlement);
        env.events().publish(
            (Symbol::new(&env, "settlement_approved"), settlement_id),
            settlement.clone(),
        );
        settlement
    }

    pub fn execute_settlement(env: Env, settlement_id: u64, token_contract: Address) {
        Self::require_not_paused(&env);
        let mut settlement: Settlement = env
            .storage()
            .persistent()
            .get(&DataKey::Settlement(settlement_id))
            .unwrap();
        if settlement.status != SettlementStatus::Pending {
            panic!("AlreadyExecuted");
        }
        let threshold: u32 = env.storage().instance().get(&DataKey::Threshold).unwrap();
        if approval_weight(&env, &settlement) < threshold {
            panic!("ThresholdNotMet");
        }
        let treasury = env.current_contract_address();
        let token_client = token::Client::new(&env, &token_contract);
        token_client.transfer(&treasury, &settlement.merchant_address, &settlement.amount);
        settlement.status = SettlementStatus::Executed;
        env.storage()
            .persistent()
            .set(&DataKey::Settlement(settlement_id), &settlement);
        env.events().publish(
            (Symbol::new(&env, "settlement_executed"), settlement_id),
            settlement,
        );
    }

    pub fn get_pending_settlements(env: Env) -> Vec<Settlement> {
        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::SettlementCount)
            .unwrap_or(0);
        let mut pending = Vec::new(&env);
        let mut id = 1;
        while id <= count {
            let settlement: Settlement = env
                .storage()
                .persistent()
                .get(&DataKey::Settlement(id))
                .unwrap();
            if settlement.status == SettlementStatus::Pending {
                pending.push_back(settlement);
            }
            id += 1;
        }
        pending
    }

    pub fn pause(env: Env, admin: Address) {
        Self::require_admin(&env, &admin);
        env.storage().instance().set(&DataKey::Paused, &true);
    }

    pub fn unpause(env: Env, admin: Address) {
        Self::require_admin(&env, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
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
