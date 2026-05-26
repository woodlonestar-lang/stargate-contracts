use soroban_sdk::{testutils::Address as _, Address, Env};
use treasury::{SettlementStatus, TreasuryContract, TreasuryContractClient};

#[test]
fn approvals_accumulate_until_threshold() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let backup = Address::generate(&env);
    let merchant = Address::generate(&env);
    let id = env.register_contract(None, TreasuryContract);
    let client = TreasuryContractClient::new(&env, &id);
    client.initialize(&admin, &2);
    client.set_signer(&admin, &backup, &1);
    let settlement_id = client.propose_settlement(&admin, &merchant, &10_000_000);
    let settlement = client.approve_settlement(&backup, &settlement_id);
    assert_eq!(settlement.status, SettlementStatus::Pending);
    assert_eq!(settlement.approvals.len(), 2);
}

#[test]
fn authorized_caller_can_pause() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let id = env.register_contract(None, TreasuryContract);
    let client = TreasuryContractClient::new(&env, &id);
    client.initialize(&admin, &2);

    client.pause(&admin);
}

#[test]
fn authorized_caller_can_unpause() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let id = env.register_contract(None, TreasuryContract);
    let client = TreasuryContractClient::new(&env, &id);
    client.initialize(&admin, &2);

    client.pause(&admin);
    client.unpause(&admin);
}

#[test]
fn guarded_function_succeeds_after_unpause() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    let merchant = Address::generate(&env);
    let id = env.register_contract(None, TreasuryContract);
    let client = TreasuryContractClient::new(&env, &id);
    client.initialize(&admin, &2);
    client.set_signer(&admin, &signer, &1);

    // Create a settlement before pausing
    let settlement_id = client.propose_settlement(&signer, &merchant, &10_000_000);
    assert_eq!(settlement_id, 1);

    // Pause, then unpause
    client.pause(&admin);
    client.unpause(&admin);

    // Verify settlement operations work after unpause
    let settlement_id2 = client.propose_settlement(&signer, &merchant, &20_000_000);
    assert_eq!(settlement_id2, 2);
}

#[test]
fn dispute_can_be_raised_against_settlement() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    let merchant = Address::generate(&env);
    let claimant = Address::generate(&env);
    let id = env.register_contract(None, TreasuryContract);
    let client = TreasuryContractClient::new(&env, &id);
    client.initialize(&admin, &2);
    client.set_signer(&admin, &signer, &1);

    let settlement_id = client.propose_settlement(&signer, &merchant, &10_000_000);

    let dispute_id = client.raise_dispute(&claimant, &settlement_id, &merchant, &5_000_000);
    assert_eq!(dispute_id, 1);
}

#[test]
fn dispute_resolved_in_favor_of_claimant() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    let merchant = Address::generate(&env);
    let claimant = Address::generate(&env);
    let id = env.register_contract(None, TreasuryContract);
    let client = TreasuryContractClient::new(&env, &id);
    client.initialize(&admin, &2);
    client.set_signer(&admin, &signer, &1);

    let settlement_id = client.propose_settlement(&signer, &merchant, &10_000_000);
    let dispute_id = client.raise_dispute(&claimant, &settlement_id, &merchant, &5_000_000);

    client.resolve_dispute(&admin, &dispute_id, &true);
}

#[test]
fn dispute_resolved_in_favor_of_counterparty() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    let merchant = Address::generate(&env);
    let claimant = Address::generate(&env);
    let id = env.register_contract(None, TreasuryContract);
    let client = TreasuryContractClient::new(&env, &id);
    client.initialize(&admin, &2);
    client.set_signer(&admin, &signer, &1);

    let settlement_id = client.propose_settlement(&signer, &merchant, &10_000_000);
    let dispute_id = client.raise_dispute(&claimant, &settlement_id, &merchant, &5_000_000);

    client.resolve_dispute(&admin, &dispute_id, &false);
}
