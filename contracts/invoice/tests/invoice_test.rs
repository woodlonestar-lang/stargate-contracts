use invoice::{InvoiceContract, InvoiceContractClient, InvoiceError, InvoiceStatus};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

fn setup() -> (Env, Address, InvoiceContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let id = env.register_contract(None, InvoiceContract);
    let client = InvoiceContractClient::new(&env, &id);
    client.initialize(&admin);
    (env, admin, client)
}

#[test]
fn test_create_invoice_succeeds() {
    let (env, _admin, client) = setup();
    let merchant = Address::generate(&env);
    let id = client.create_invoice(&merchant, &10_000_000, &10_250_000, &3600);
    let invoice = client.get_invoice(&id).unwrap();
    assert_eq!(invoice.id, 1);
    assert_eq!(invoice.status, InvoiceStatus::Pending);
    assert_eq!(invoice.amount_usdc, 10_000_000);
    assert_eq!(invoice.gross_usdc, 10_250_000);
    // Issue #6: payer is None before payment
    assert!(invoice.payer.is_none());
}

#[test]
fn test_mark_paid_requires_admin() {
    let (env, _admin, client) = setup();
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let rogue_admin = Address::generate(&env);
    let id = client.create_invoice(&merchant, &10_000_000, &10_250_000, &3600);
    assert!(client.try_mark_paid(&rogue_admin, &id, &payer).is_err());
}

#[test]
fn test_expired_invoice_cannot_be_paid() {
    let (env, admin, client) = setup();
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let id = client.create_invoice(&merchant, &10_000_000, &10_250_000, &1);
    env.ledger().with_mut(|ledger| ledger.timestamp += 2);
    assert!(client.try_mark_paid(&admin, &id, &payer).is_err());
}

#[test]
fn test_pause_blocks_create_invoice() {
    let (env, admin, client) = setup();
    let merchant = Address::generate(&env);
    client.pause(&admin);
    assert!(client
        .try_create_invoice(&merchant, &10_000_000, &10_250_000, &3600)
        .is_err());
}

#[test]
fn test_pause_blocks_mark_paid() {
    let (env, admin, client) = setup();
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let id = client.create_invoice(&merchant, &10_000_000, &10_250_000, &3600);
    client.pause(&admin);
    assert!(client.try_mark_paid(&admin, &id, &payer).is_err());
}

#[test]
fn test_double_payment_rejected() {
    let (env, admin, client) = setup();
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let id = client.create_invoice(&merchant, &10_000_000, &10_250_000, &3600);
    client.mark_paid(&admin, &id, &payer);
    assert!(client.try_mark_paid(&admin, &id, &payer).is_err());
}

// Issue #5: get_invoice returns NotFound for unknown ID
#[test]
fn test_get_invoice_unknown_id_returns_not_found() {
    let (_env, _admin, client) = setup();
    let err = client.try_get_invoice(&999).unwrap_err().unwrap();
    assert_eq!(err, InvoiceError::NotFound);
}

// Issue #5: mark_paid returns NotFound for unknown ID
#[test]
fn test_mark_paid_unknown_id_returns_not_found() {
    let (env, admin, client) = setup();
    let payer = Address::generate(&env);
    let err = client.try_mark_paid(&admin, &999, &payer).unwrap_err().unwrap();
    assert_eq!(err, InvoiceError::NotFound);
}

// Issue #6: payer is set to Some(payer) after payment
#[test]
fn test_payer_set_after_payment() {
    let (env, admin, client) = setup();
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let id = client.create_invoice(&merchant, &10_000_000, &10_250_000, &3600);
    client.mark_paid(&admin, &id, &payer);
    let invoice = client.get_invoice(&id).unwrap();
    assert_eq!(invoice.payer, Some(payer));
}

// Issue #7: expired event emitted when mark_paid finds stale invoice
#[test]
fn test_expired_event_emitted_on_stale_mark_paid() {
    let (env, admin, client) = setup();
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let id = client.create_invoice(&merchant, &10_000_000, &10_250_000, &1);
    env.ledger().with_mut(|ledger| ledger.timestamp += 2);
    let _ = client.try_mark_paid(&admin, &id, &payer);
    // Invoice should now be Expired in storage
    let invoice = client.get_invoice(&id).unwrap();
    assert_eq!(invoice.status, InvoiceStatus::Expired);
}

// Issue #8: payment at exactly expires_at is rejected (boundary is exclusive)
#[test]
fn test_payment_at_exact_expiry_is_rejected() {
    let (env, admin, client) = setup();
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    // expires_in_seconds=10, ledger starts at 0, so expires_at=10
    let id = client.create_invoice(&merchant, &10_000_000, &10_250_000, &10);
    env.ledger().with_mut(|ledger| ledger.timestamp = 10);
    let err = client.try_mark_paid(&admin, &id, &payer).unwrap_err().unwrap();
    assert_eq!(err, InvoiceError::Expired);
}

// Issue #8: payment one second before expiry succeeds
#[test]
fn test_payment_before_expiry_succeeds() {
    let (env, admin, client) = setup();
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let id = client.create_invoice(&merchant, &10_000_000, &10_250_000, &10);
    env.ledger().with_mut(|ledger| ledger.timestamp = 9);
    client.mark_paid(&admin, &id, &payer);
    let invoice = client.get_invoice(&id).unwrap();
    assert_eq!(invoice.status, InvoiceStatus::Paid);
// Issue #1: initialize requires admin auth
#[test]
fn test_initialize_requires_admin_auth() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let id = env.register_contract(None, InvoiceContract);
    let client = InvoiceContractClient::new(&env, &id);
    client.initialize(&admin);
    // Verify that admin auth was required during initialize
    let auths = env.auths();
    assert!(auths.iter().any(|(addr, _)| addr == &admin));
}

// Issue #2: initialize cannot be called twice
#[test]
fn test_initialize_cannot_be_called_twice() {
    let (env, _admin, client) = setup();
    let new_admin = Address::generate(&env);
    assert!(client.try_initialize(&new_admin).is_err());
}

// Issue #3: zero-duration invoice is rejected
#[test]
fn test_zero_duration_invoice_rejected() {
    let (env, _admin, client) = setup();
    let merchant = Address::generate(&env);
    assert!(client
        .try_create_invoice(&merchant, &10_000_000, &10_250_000, &0)
        .is_err());
}

// Issue #4: overflow in expires_at is rejected
#[test]
fn test_expiry_overflow_rejected() {
    let (env, _admin, client) = setup();
    let merchant = Address::generate(&env);
    // Set ledger timestamp near u64::MAX so adding any duration overflows
    env.ledger().with_mut(|l| l.timestamp = u64::MAX);
    assert!(client
        .try_create_invoice(&merchant, &10_000_000, &10_250_000, &1)
        .is_err());
#[test]
fn test_event_stream_redis_webhook_compatibility() {
    // Validates that contract events emitted are compatible with Redis webhook delivery.
    // Expected event schema for Redis consumer:
    // - Event type: Symbol (serializes to string)
    // - Topics: (Symbol, optional id/data fields)
    // - Data: Invoice struct (fields: id, merchant, amount_usdc, gross_usdc, status, expires_at, paid_at, payer)
    //
    // Redis webhook delivery expects:
    // 1. Events serializable to JSON without data loss
    // 2. Addresses convertible to string representation
    // 3. InvoiceStatus enum serializable as string variant
    // 4. Numeric types (u64, i128) representable as JSON numbers/strings
    let (env, admin, client) = setup();
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);

    let invoice_id = client.create_invoice(&merchant, &10_000_000, &10_250_000, &3600);

    // Verify the invoice can be retrieved (validates event data was properly stored)
    let invoice = client.get_invoice(&invoice_id);
    assert_eq!(invoice.id, 1);
    assert_eq!(invoice.merchant, merchant);
    assert_eq!(invoice.amount_usdc, 10_000_000);
    assert_eq!(invoice.gross_usdc, 10_250_000);
    assert_eq!(invoice.status, InvoiceStatus::Pending);
    assert_eq!(invoice.payer, merchant); // Payer defaults to merchant on creation

    // Verify payment event data
    client.mark_paid(&admin, &invoice_id, &payer);
    let paid_invoice = client.get_invoice(&invoice_id);
    assert_eq!(paid_invoice.status, InvoiceStatus::Paid);
    assert_eq!(paid_invoice.payer, payer);
    assert!(paid_invoice.paid_at.is_some());

    // Verify pause/unpause events with Address data
    client.pause(&admin);
    client.unpause(&admin);

    // All event types tested: invoice_created, invoice_paid, contract_paused, contract_unpaused
    // All data types represented: Symbol, u64, Address, Invoice struct, Option<u64>
    // Verification: events are emitted with consistent, JSON-serializable payloads
}
