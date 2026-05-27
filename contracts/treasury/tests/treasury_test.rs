use soroban_sdk::{testutils::Address as _, Address, Env};
use treasury::{SettlementStatus, TreasuryContract, TreasuryContractClient};

fn setup(env: &Env, threshold: u32) -> (TreasuryContractClient, Address, Address) {
    env.mock_all_auths();
    let admin = Address::generate(env);
    let id = env.register_contract(None, TreasuryContract);
    let client = TreasuryContractClient::new(env, &id);
    client.initialize(&admin, &threshold);
    (client, admin, id)
}

// Original test — approvals accumulate until threshold
#[test]
fn approvals_accumulate_until_threshold() {
fn setup() -> (Env, Address, Address, TreasuryContractClient<'static>) {
    let env = Env::default();
    let (client, admin, _) = setup(&env, 2);
    let backup = Address::generate(&env);
    let merchant = Address::generate(&env);
    let id = env.register_contract(None, TreasuryContract);
    let client = TreasuryContractClient::new(&env, &id);
    client.initialize(&admin, &2);
    client.set_signer(&admin, &backup, &1);
    (env, admin, backup, client)
}

#[test]
fn approvals_accumulate_until_threshold() {
    let (env, admin, backup, client) = setup();
    let merchant = Address::generate(&env);
    let settlement_id = client.propose_settlement(&admin, &merchant, &10_000_000);
    let settlement = client.approve_settlement(&backup, &settlement_id);
    assert_eq!(settlement.status, SettlementStatus::Pending);
    assert_eq!(settlement.approvals.len(), 2);
    // approval_weight should be 2 (admin=1 + backup=1)
    assert_eq!(settlement.approval_weight, 2);
}

// Fix #13: approve_settlement on missing ID should panic with SettlementNotFound
#[test]
#[should_panic(expected = "SettlementNotFound")]
fn approve_missing_settlement_returns_typed_error() {
    let env = Env::default();
    let (client, _, _) = setup(&env, 2);
    let signer = Address::generate(&env);
    client.approve_settlement(&signer, &999);
}

// Fix #13: execute_settlement on missing ID should panic with SettlementNotFound
#[test]
#[should_panic(expected = "SettlementNotFound")]
fn execute_missing_settlement_returns_typed_error() {
    let env = Env::default();
    let (client, _, _) = setup(&env, 2);
    let token = Address::generate(&env);
    client.execute_settlement(&999, &token);
}

// Fix #15: weight snapshotted at approval time — changing weight after approval
// does not affect the stored approval_weight
#[test]
fn signer_weight_change_after_approval_does_not_affect_snapshot() {
    let env = Env::default();
    let (client, admin, _) = setup(&env, 2);
    let backup = Address::generate(&env);
    let merchant = Address::generate(&env);
    client.set_signer(&admin, &backup, &1);
    let sid = client.propose_settlement(&admin, &merchant, &10_000_000);
    client.approve_settlement(&backup, &sid);
    // Now reduce backup weight to 0 — snapshotted weight should remain 2
    client.set_signer(&admin, &backup, &0);
    let pending = client.get_pending_settlements();
    assert_eq!(pending.get(0).unwrap().approval_weight, 2);
}

// Fix #16: execute should panic when threshold is zero (invalid)
// We test this by re-initializing with threshold=0
#[test]
#[should_panic(expected = "ThresholdNotConfigured")]
fn execute_rejects_zero_threshold() {
    let env = Env::default();
    let (client, admin, _) = setup(&env, 0);
    let merchant = Address::generate(&env);
    let token = Address::generate(&env);
    let sid = client.propose_settlement(&admin, &merchant, &10_000_000);
    client.execute_settlement(&sid, &token);
}

// Fix #17: execute should panic when token_contract is the treasury contract itself
#[test]
#[should_panic(expected = "InvalidTokenContract")]
fn execute_rejects_self_as_token_contract() {
    let env = Env::default();
    let (client, admin, contract_id) = setup(&env, 1);
    let merchant = Address::generate(&env);
    let sid = client.propose_settlement(&admin, &merchant, &10_000_000);
    client.execute_settlement(&sid, &contract_id);
}

#[test]
fn pause_and_unpause_emit_events() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let id = env.register_contract(None, TreasuryContract);
    let client = TreasuryContractClient::new(&env, &id);
    client.initialize(&admin, &1);
    client.pause(&admin);
    client.unpause(&admin);
    // after unpause, proposals work again
    let settlement_id = client.propose_settlement(&admin, &merchant, &1_000);
    assert_eq!(settlement_id, 1);
fn execute_settlement_requires_authorized_signer() {
    let (env, admin, backup, client) = setup();
    let merchant = Address::generate(&env);
    let rogue = Address::generate(&env);
    let settlement_id = client.propose_settlement(&admin, &merchant, &10_000_000);
    client.approve_settlement(&backup, &settlement_id);
    let token = env.register_contract(None, TreasuryContract);
    assert!(client
        .try_execute_settlement(&rogue, &settlement_id, &token)
        .is_err());
}
