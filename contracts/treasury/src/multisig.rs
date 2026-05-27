use soroban_sdk::{contracterror, contracttype, Address, Vec};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum TreasuryError {
    AlreadyInitialized = 1,
    ZeroThreshold = 2,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SettlementStatus {
    Pending,
    Executed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Settlement {
    pub id: u64,
    pub merchant_address: Address,
    pub amount: i128,
    pub approvals: Vec<Address>,
    pub approval_weight: u32,
    pub status: SettlementStatus,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Threshold,
    SettlementCount,
    Settlement(u64),
    Signer(Address),
    Paused,
}
