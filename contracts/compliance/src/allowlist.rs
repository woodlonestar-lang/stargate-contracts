use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    Allowed(Address),
    Blocked(Address),
    AllowedUntil(Address),
    Paused,
}
