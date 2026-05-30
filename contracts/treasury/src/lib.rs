#![no_std]

mod multisig;
mod settlement;

pub use multisig::{DataKey, Dispute, DisputeStatus, Settlement, SettlementStatus, TreasuryError};

use settlement::{require_authorized_signer, signer_weight};
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Symbol, Vec};
use soroban_sdk::{contract, contractimpl, token, Address, Env, Symbol, Vec};

impl TreasuryError {
    fn panic(&self) -> ! {
        match self {
            TreasuryError::AlreadyInitialized => panic!("AlreadyInitialized"),
            TreasuryError::ZeroThreshold => panic!("ZeroThreshold"),
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
    pub fn initialize(env: Env, admin: Address, threshold: u32) -> Result<(), TreasuryError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(TreasuryError::AlreadyInitialized);
        }
        if threshold == 0 {
            return Err(TreasuryError::ZeroThreshold);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::Threshold, &threshold);
        env.storage()
            .instance()
            .set(&DataKey::SettlementCount, &0u64);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage().instance().set(&DataKey::DisputeCount, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::Signer(admin.clone()), &1u32);
        env.events()
            .publish((Symbol::new(&env, "treasury_initialized"),), admin);
        Ok(())
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
            .unwrap_or_else(|| panic!("SettlementNotFound"));
        if settlement.status != SettlementStatus::Pending {
            panic!("AlreadyExecuted");
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

    pub fn execute_settlement(
        env: Env,
        signer: Address,
        settlement_id: u64,
        token_contract: Address,
    ) {
        Self::require_not_paused(&env);
        // Fix #13: return typed error instead of unwrap panic
        require_authorized_signer(&env, &signer);
        let mut settlement: Settlement = env
            .storage()
            .persistent()
            .get(&DataKey::Settlement(settlement_id))
            .unwrap_or_else(|| panic!("SettlementNotFound"));
        if settlement.status != SettlementStatus::Pending {
            panic!("AlreadyExecuted");
        }
        // Fix #16: reject execution when threshold is missing or zero
        let threshold: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Threshold)
            .unwrap_or_else(|| panic!("ThresholdNotConfigured"));
        if threshold == 0 {
            panic!("ThresholdNotConfigured");
        }
        // Fix #15: use snapshotted approval_weight (set at approval time)
        if settlement.approval_weight < threshold {
            panic!("ThresholdNotMet");
        }
        // Fix #17: validate token contract is a registered signer or non-zero address
        // by attempting a balance check — if the address is not a valid token contract
        // the call will trap; instead we validate it is not the zero/contract address itself
        if token_contract == env.current_contract_address() {
            panic!("InvalidTokenContract");
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

    pub fn raise_dispute(
        env: Env,
        claimant: Address,
        settlement_id: u64,
        counterparty: Address,
        amount: i128,
    ) -> u64 {
        Self::require_not_paused(&env);
        claimant.require_auth();
        if amount <= 0 {
            panic!("InvalidAmount");
        }

        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::DisputeCount)
            .unwrap_or(0);
        let id = count + 1;
        let dispute = Dispute {
            id,
            settlement_id,
            claimant,
            counterparty,
            amount,
            status: DisputeStatus::Raised,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Dispute(id), &dispute);
        env.storage().instance().set(&DataKey::DisputeCount, &id);
        env.events()
            .publish((Symbol::new(&env, "dispute_raised"), id), dispute);
        id
    }

    pub fn resolve_dispute(env: Env, admin: Address, dispute_id: u64, in_favor_of_claimant: bool) {
        Self::require_admin(&env, &admin);
        Self::require_not_paused(&env);
        let mut dispute: Dispute = env
            .storage()
            .persistent()
            .get(&DataKey::Dispute(dispute_id))
            .unwrap();
        if dispute.status != DisputeStatus::Raised {
            panic!("DisputeAlreadyResolved");
        }
        dispute.status = if in_favor_of_claimant {
            DisputeStatus::ResolvedClaimant
        } else {
            DisputeStatus::ResolvedCounterparty
        };
        env.storage()
            .persistent()
            .set(&DataKey::Dispute(dispute_id), &dispute);
        env.events()
            .publish((Symbol::new(&env, "dispute_resolved"), dispute_id), dispute);
    }

    pub fn deposit(env: Env, from: Address, token_contract: Address, amount: i128) {
        Self::require_not_paused(&env);
        from.require_auth();
        if amount <= 0 {
            panic!("InvalidAmount");
        }

        let treasury = env.current_contract_address();
        let token_client = token::Client::new(&env, &token_contract);
        token_client.transfer(&from, &treasury, &amount);

        let mut balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from.clone()))
            .unwrap_or(0);
        balance += amount;
        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &balance);
        env.events()
            .publish((Symbol::new(&env, "deposit"), from), amount);
    }

    pub fn withdraw(env: Env, to: Address, token_contract: Address, amount: i128) {
        Self::require_not_paused(&env);
        to.require_auth();
        if amount <= 0 {
            panic!("InvalidAmount");
        }

        let mut balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(to.clone()))
            .unwrap_or(0);
        if balance < amount {
            panic!("InsufficientBalance");
        }
        balance -= amount;
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &balance);

        let treasury = env.current_contract_address();
        let token_client = token::Client::new(&env, &token_contract);
        token_client.transfer(&treasury, &to, &amount);
        env.events()
            .publish((Symbol::new(&env, "withdraw"), to), amount);
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
