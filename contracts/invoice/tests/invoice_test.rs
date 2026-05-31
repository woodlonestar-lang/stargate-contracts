use invoice::{
    InvoiceContract, InvoiceContractClient, InvoiceError, InvoiceStatus, MaybeAddress, MaybeBytes,
};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

extern crate std;
use std::{collections::HashSet, fs, path::Path};

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
    let id = client.create_invoice(
        &merchant,
        &10_000_000,
        &10_250_000,
        &3600,
        &MaybeBytes::None,
        &MaybeBytes::None,
    );
    let invoice = client.get_invoice(&id).unwrap();
    assert_eq!(invoice.id, 1);
    assert_eq!(invoice.status, InvoiceStatus::Pending);
    assert_eq!(invoice.amount_usdc, 10_000_000);
    assert_eq!(invoice.gross_usdc, 10_250_000);
    assert_eq!(invoice.payer, MaybeAddress::None);
}

#[test]
fn test_mark_paid_requires_admin() {
    let (env, _admin, client) = setup();
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let rogue_admin = Address::generate(&env);
    let id = client.create_invoice(
        &merchant,
        &10_000_000,
        &10_250_000,
        &3600,
        &MaybeBytes::None,
        &MaybeBytes::None,
    );
    assert!(client.try_mark_paid(&rogue_admin, &id, &payer).is_err());
}

#[test]
fn test_expired_invoice_cannot_be_paid() {
    let (env, admin, client) = setup();
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let id = client.create_invoice(
        &merchant,
        &10_000_000,
        &10_250_000,
        &1,
        &MaybeBytes::None,
        &MaybeBytes::None,
    );
    env.ledger().with_mut(|ledger| ledger.timestamp += 2);
    assert!(client.try_mark_paid(&admin, &id, &payer).is_err());
}

#[test]
fn test_pause_blocks_create_invoice() {
    let (env, admin, client) = setup();
    let merchant = Address::generate(&env);
    client.pause(&admin);
    assert!(client
        .try_create_invoice(
            &merchant,
            &10_000_000,
            &10_250_000,
            &3600,
            &MaybeBytes::None,
            &MaybeBytes::None
        )
        .is_err());
}

#[test]
fn test_pause_blocks_mark_paid() {
    let (env, admin, client) = setup();
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let id = client.create_invoice(
        &merchant,
        &10_000_000,
        &10_250_000,
        &3600,
        &MaybeBytes::None,
        &MaybeBytes::None,
    );
    client.pause(&admin);
    assert!(client.try_mark_paid(&admin, &id, &payer).is_err());
}

#[test]
fn test_double_payment_rejected() {
    let (env, admin, client) = setup();
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let id = client.create_invoice(
        &merchant,
        &10_000_000,
        &10_250_000,
        &3600,
        &MaybeBytes::None,
        &MaybeBytes::None,
    );
    client.mark_paid(&admin, &id, &payer);
    assert!(client.try_mark_paid(&admin, &id, &payer).is_err());
}

#[test]
fn test_get_invoice_unknown_id_returns_not_found() {
    let (_env, _admin, client) = setup();
    let err = client.try_get_invoice(&999).unwrap_err().unwrap();
    assert_eq!(err, InvoiceError::NotFound);
}

#[test]
fn test_mark_paid_unknown_id_returns_not_found() {
    let (env, admin, client) = setup();
    let payer = Address::generate(&env);
    let err = client
        .try_mark_paid(&admin, &999, &payer)
        .unwrap_err()
        .unwrap();
    assert_eq!(err, InvoiceError::NotFound);
}

#[test]
fn test_payer_set_after_payment() {
    let (env, admin, client) = setup();
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let id = client.create_invoice(
        &merchant,
        &10_000_000,
        &10_250_000,
        &3600,
        &MaybeBytes::None,
        &MaybeBytes::None,
    );
    client.mark_paid(&admin, &id, &payer);
    let invoice = client.get_invoice(&id).unwrap();
    assert_eq!(invoice.payer, MaybeAddress::Some(payer));
}

#[test]
fn test_expired_event_emitted_on_stale_mark_paid() {
    let (env, admin, client) = setup();
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let id = client.create_invoice(
        &merchant,
        &10_000_000,
        &10_250_000,
        &1,
        &MaybeBytes::None,
        &MaybeBytes::None,
    );
    env.ledger().with_mut(|ledger| ledger.timestamp += 2);
    let err = client
        .try_mark_paid(&admin, &id, &payer)
        .unwrap_err()
        .unwrap();
    assert_eq!(err, InvoiceError::Expired);
    // Storage is rolled back on error; invoice remains Pending
    let invoice = client.get_invoice(&id).unwrap();
    assert_eq!(invoice.status, InvoiceStatus::Pending);
}

// Payment at exactly expires_at is rejected — the boundary is exclusive.
// expires_in_seconds=10, ledger starts at 0, so expires_at=10.
// Setting timestamp=10 means now >= expires_at → Expired.
#[test]
fn test_payment_at_exact_expiry_is_rejected() {
    let (env, admin, client) = setup();
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let id = client.create_invoice(
        &merchant,
        &10_000_000,
        &10_250_000,
        &10,
        &MaybeBytes::None,
        &MaybeBytes::None,
    );
    env.ledger().with_mut(|ledger| ledger.timestamp = 10);
    let err = client
        .try_mark_paid(&admin, &id, &payer)
        .unwrap_err()
        .unwrap();
    assert_eq!(err, InvoiceError::Expired);
}

// Payment one second before expires_at succeeds — last valid moment is expires_at - 1.
#[test]
fn test_payment_before_expiry_succeeds() {
    let (env, admin, client) = setup();
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let id = client.create_invoice(
        &merchant,
        &10_000_000,
        &10_250_000,
        &10,
        &MaybeBytes::None,
        &MaybeBytes::None,
    );
    env.ledger().with_mut(|ledger| ledger.timestamp = 9);
    client.mark_paid(&admin, &id, &payer);
    let invoice = client.get_invoice(&id).unwrap();
    assert_eq!(invoice.status, InvoiceStatus::Paid);
}

#[test]
fn test_initialize_requires_admin_auth() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let id = env.register_contract(None, InvoiceContract);
    let client = InvoiceContractClient::new(&env, &id);
    client.initialize(&admin);
    let auths = env.auths();
    assert!(auths.iter().any(|(addr, _)| addr == &admin));
}

#[test]
fn test_initialize_cannot_be_called_twice() {
    let (env, _admin, client) = setup();
    let new_admin = Address::generate(&env);
    assert!(client.try_initialize(&new_admin).is_err());
}

#[test]
fn test_zero_duration_invoice_rejected() {
    let (env, _admin, client) = setup();
    let merchant = Address::generate(&env);
    assert!(client
        .try_create_invoice(
            &merchant,
            &10_000_000,
            &10_250_000,
            &0,
            &MaybeBytes::None,
            &MaybeBytes::None
        )
        .is_err());
}

#[test]
fn test_expiry_overflow_rejected() {
    let (env, _admin, client) = setup();
    let merchant = Address::generate(&env);
    env.ledger().with_mut(|l| l.timestamp = u64::MAX);
    assert!(client
        .try_create_invoice(
            &merchant,
            &10_000_000,
            &10_250_000,
            &1,
            &MaybeBytes::None,
            &MaybeBytes::None
        )
        .is_err());
}

#[test]
fn test_event_stream_redis_webhook_compatibility() {
    let (env, admin, client) = setup();
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);

    let invoice_id = client.create_invoice(
        &merchant,
        &10_000_000,
        &10_250_000,
        &3600,
        &MaybeBytes::None,
        &MaybeBytes::None,
    );

    let invoice = client.get_invoice(&invoice_id).unwrap();
    assert_eq!(invoice.id, 1);
    assert_eq!(invoice.merchant, merchant);
    assert_eq!(invoice.amount_usdc, 10_000_000);
    assert_eq!(invoice.gross_usdc, 10_250_000);
    assert_eq!(invoice.status, InvoiceStatus::Pending);
    assert_eq!(invoice.payer, MaybeAddress::None);

    client.mark_paid(&admin, &invoice_id, &payer);
    let paid_invoice = client.get_invoice(&invoice_id).unwrap();
    assert_eq!(paid_invoice.status, InvoiceStatus::Paid);
    assert_eq!(paid_invoice.payer, MaybeAddress::Some(payer));
    assert!(paid_invoice.paid_at.is_some());

    client.pause(&admin);
    client.unpause(&admin);
}

// ABI snapshot comparison: asserts abis/invoice.json stays in sync with the
// contract's public surface. Run via `cargo test` or `make check-abi-snapshots`.
#[test]
fn test_abi_snapshot_matches_contract() {
    // Canonical function and event lists derived from lib.rs / events.rs.
    let expected_functions: HashSet<&str> = [
        "initialize",
        "create_invoice",
        "mark_paid",
        "get_invoice",
        "get_invoice_status",
        "cancel_invoice",
        "request_refund",
        "batch_expire",
        "pause",
        "unpause",
    ]
    .iter()
    .copied()
    .collect();

    let expected_events: HashSet<&str> = [
        "invoice_created",
        "invoice_paid",
        "invoice_expired",
        "invoice_cancelled",
        "invoice_refund_req",
        "contract_paused",
        "contract_unpaused",
    ]
    .iter()
    .copied()
    .collect();

    // Locate abis/invoice.json relative to the workspace root (CARGO_MANIFEST_DIR
    // points to contracts/invoice; walk up two levels).
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let abi_path = manifest_dir.join("../../abis/invoice.json");
    let raw = fs::read_to_string(&abi_path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", abi_path.display()));

    // --- functions ---
    let fns_block = raw
        .split("\"functions\"")
        .nth(1)
        .expect("\"functions\" key missing from abis/invoice.json");
    let fns_array = &fns_block[fns_block.find('[').unwrap()..=fns_block.find(']').unwrap()];
    let snapshot_functions: HashSet<&str> = fns_array
        .split('"')
        .filter(|s| !s.trim().is_empty() && !s.contains('[') && !s.contains(']'))
        .collect();

    // --- events ---
    let evts_block = raw
        .split("\"events\"")
        .nth(1)
        .expect("\"events\" key missing from abis/invoice.json");
    let evts_array = &evts_block[evts_block.find('[').unwrap()..=evts_block.find(']').unwrap()];
    let snapshot_events: HashSet<&str> = evts_array
        .split('"')
        .filter(|s| !s.trim().is_empty() && !s.contains('[') && !s.contains(']'))
        .collect();

    assert_eq!(
        snapshot_functions,
        expected_functions,
        "abis/invoice.json functions list is out of sync with the contract.\n\
         Missing from snapshot : {:?}\n\
         Extra in snapshot     : {:?}\n\
         Run `make update-abi-snapshots` to regenerate.",
        expected_functions
            .difference(&snapshot_functions)
            .collect::<Vec<_>>(),
        snapshot_functions
            .difference(&expected_functions)
            .collect::<Vec<_>>(),
    );

    assert_eq!(
        snapshot_events,
        expected_events,
        "abis/invoice.json events list is out of sync with the contract.\n\
         Missing from snapshot : {:?}\n\
         Extra in snapshot     : {:?}\n\
         Run `make update-abi-snapshots` to regenerate.",
        expected_events
            .difference(&snapshot_events)
            .collect::<Vec<_>>(),
        snapshot_events
            .difference(&expected_events)
            .collect::<Vec<_>>(),
    );
}
