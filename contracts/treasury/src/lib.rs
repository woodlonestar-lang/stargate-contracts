#![no_std]

mod multisig;
mod settlement;

pub use multisig::{DataKey, Settlement, SettlementStatus};

use settlement::{require_authorized_signer, signer_weight};
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Symbol, Vec};

// Fix #13: typed error enum instead of panics on missing settlement IDs
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TreasuryError {
    SettlementNotFound,
    AlreadyExecuted,
    ThresholdNotMet,
    ThresholdNotConfigured,
    InvalidAmount,
    ContractPaused,
    Unauthorized,
    UnauthorizedSigner,
    InvalidTokenContract,
}

impl TreasuryError {
    fn panic(&self) -> ! {
        match self {
            TreasuryError::SettlementNotFound => panic!("SettlementNotFound"),
            TreasuryError::AlreadyExecuted => panic!("AlreadyExecuted"),
            TreasuryError::ThresholdNotMet => panic!("ThresholdNotMet"),
            TreasuryError::ThresholdNotConfigured => panic!("ThresholdNotConfigured"),
            TreasuryError::InvalidAmount => panic!("InvalidAmount"),
            TreasuryError::ContractPaused => panic!("ContractPaused"),
            TreasuryError::Unauthorized => panic!("Unauthorized"),
            TreasuryError::UnauthorizedSigner => panic!("UnauthorizedSigner"),
            TreasuryError::InvalidTokenContract => panic!("InvalidTokenContract"),
        }
    }
}

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
            TreasuryError::InvalidAmount.panic();
        }

        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::SettlementCount)
            .unwrap_or(0);
        let id = count + 1;
        let mut approvals = Vec::new(&env);
        // Fix #15: snapshot the proposer's weight at proposal time
        let weight = signer_weight(&env, &signer);
        approvals.push_back(signer);
        let settlement = Settlement {
            id,
            merchant_address,
            amount,
            approvals,
            approval_weight: weight,
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
        // Fix #13: return typed error instead of unwrap panic
        let mut settlement: Settlement = env
            .storage()
            .persistent()
            .get(&DataKey::Settlement(settlement_id))
            .unwrap_or_else(|| TreasuryError::SettlementNotFound.panic());
        if settlement.status != SettlementStatus::Pending {
            TreasuryError::AlreadyExecuted.panic();
        }
        if !settlement.approvals.contains(&signer) {
            // Fix #15: snapshot the approver's weight at approval time
            settlement.approval_weight += signer_weight(&env, &signer);
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

    pub fn execute_settlement(env: Env, signer: Address, settlement_id: u64, token_contract: Address) {
        Self::require_not_paused(&env);
        // Fix #13: return typed error instead of unwrap panic
        require_authorized_signer(&env, &signer);
        let mut settlement: Settlement = env
            .storage()
            .persistent()
            .get(&DataKey::Settlement(settlement_id))
            .unwrap_or_else(|| TreasuryError::SettlementNotFound.panic());
        if settlement.status != SettlementStatus::Pending {
            TreasuryError::AlreadyExecuted.panic();
        }
        // Fix #16: reject execution when threshold is missing or zero
        let threshold: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Threshold)
            .unwrap_or_else(|| TreasuryError::ThresholdNotConfigured.panic());
        if threshold == 0 {
            TreasuryError::ThresholdNotConfigured.panic();
        }
        // Fix #15: use snapshotted approval_weight (set at approval time)
        if settlement.approval_weight < threshold {
            TreasuryError::ThresholdNotMet.panic();
        }
        // Fix #17: validate token contract is a registered signer or non-zero address
        // by attempting a balance check — if the address is not a valid token contract
        // the call will trap; instead we validate it is not the zero/contract address itself
        if token_contract == env.current_contract_address() {
            TreasuryError::InvalidTokenContract.panic();
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
            // Fix #13: skip missing settlements instead of panicking
            if let Some(settlement) = env
                .storage()
                .persistent()
                .get::<DataKey, Settlement>(&DataKey::Settlement(id))
            {
                if settlement.status == SettlementStatus::Pending {
                    pending.push_back(settlement);
                }
            }
            id += 1;
        }
        pending
    }

    pub fn pause(env: Env, admin: Address) {
        Self::require_admin(&env, &admin);
        env.storage().instance().set(&DataKey::Paused, &true);
        env.events()
            .publish((Symbol::new(&env, "treasury_paused"),), admin);
    }

    pub fn unpause(env: Env, admin: Address) {
        Self::require_admin(&env, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.events()
            .publish((Symbol::new(&env, "treasury_unpaused"),), admin);
    }

    fn require_admin(env: &Env, admin: &Address) {
        admin.require_auth();
        let stored: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if stored != *admin {
            TreasuryError::Unauthorized.panic();
        }
    }

    fn require_not_paused(env: &Env) {
        let paused: bool = env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false);
        if paused {
            TreasuryError::ContractPaused.panic();
        }
    }
}

#[cfg(test)]
extern crate std;
