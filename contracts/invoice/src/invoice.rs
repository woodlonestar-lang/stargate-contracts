use soroban_sdk::{contracterror, contracttype, Address, Bytes};

// soroban contracttype cannot store Option<Address> directly; use a tagged-union workaround
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaybeAddress {
    None,
    Some(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum InvoiceError {
    Unauthorized = 1,
    ContractPaused = 2,
    InvalidAmount = 3,
    NotPending = 4,
    Expired = 5,
    NotFound = 6,
    AlreadyInitialized = 7,
    ZeroDuration = 8,
    ExpiryOverflow = 9,
    NotPaid = 10,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InvoiceStatus {
    Pending,
    Paid,
    Expired,
    Cancelled,
    RefundRequested,
}

// contracttype enum wrappers for optional complex types; Option<Address> and
// Option<Bytes> are not supported by the contracttype macro in soroban-sdk v20.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaybeAddress {
    None,
    Some(Address),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaybeBytes {
    None,
    Some(Bytes),
}

/// `Option<Address>` is not supported by `#[contracttype]`; use this enum instead.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OptionalAddress {
    None,
    Some(Address),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Invoice {
    pub id: u64,
    pub merchant: Address,
    pub amount_usdc: i128,
    pub gross_usdc: i128,
    pub status: InvoiceStatus,
    pub expires_at: u64,
    pub paid_at: Option<u64>,
    pub payer: OptionalAddress,
    pub payer: MaybeAddress,
    pub metadata_hash: MaybeBytes,
    pub payment_link_hash: MaybeBytes,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Invoice(u64),
    InvoiceCount,
    Admin,
    Paused,
}
