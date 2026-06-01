use soroban_sdk::{contracterror, contracttype, Address};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Allowed(Address),
    Blocked(Address),
    Paused,
}

#[contracterror]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ComplianceError {
    AlreadyInitialized = 1,
}
