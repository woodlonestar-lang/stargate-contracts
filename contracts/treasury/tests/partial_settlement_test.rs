use soroban_sdk::{testutils::Address as _, Address, Env};
use treasury::{SettlementStatus, TreasuryContract, TreasuryContractClient};

fn setup(env: &Env, total: i128) -> (TreasuryContractClient, Address, Address, u64) {
    let admin = Address::generate(env);
    let merchant = Address::generate(env);
    let contract_id = env.register_contract(None, TreasuryContract);
    let client = TreasuryContractClient::new(env, &contract_id);
    // threshold=1, admin weight=1 → admin approval alone is sufficient
    client.initialize(&admin, &1);

    let token_id = env.register_stellar_asset_contract(admin.clone());
    soroban_sdk::token::StellarAssetClient::new(env, &token_id).mint(&contract_id, &total);

    let sid = client.propose_settlement(&admin, &merchant, &total);
    (client, admin, token_id, sid)
}

#[test]
fn partial_settle_deducts_amount_and_sets_partial_status() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, token_id, sid) = setup(&env, 10_000_000);

    let s = client.partial_settle(&sid, &3_000_000, &token_id);
    assert_eq!(s.amount, 7_000_000);
    assert_eq!(s.status, SettlementStatus::PartiallySettled);
}

#[test]
fn partial_settle_full_amount_marks_executed() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, token_id, sid) = setup(&env, 10_000_000);

    let s = client.partial_settle(&sid, &10_000_000, &token_id);
    assert_eq!(s.amount, 0);
    assert_eq!(s.status, SettlementStatus::Executed);
}

#[test]
fn partial_settle_twice_correctly_tracks_remainder() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, token_id, sid) = setup(&env, 10_000_000);

    client.partial_settle(&sid, &4_000_000, &token_id);
    let s = client.partial_settle(&sid, &3_000_000, &token_id);

    assert_eq!(s.amount, 3_000_000);
    assert_eq!(s.status, SettlementStatus::PartiallySettled);
}

#[test]
fn partially_settled_still_appears_in_pending_list() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, token_id, sid) = setup(&env, 10_000_000);

    client.partial_settle(&sid, &4_000_000, &token_id);
    let pending = client.get_pending_settlements();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending.get(0).unwrap().id, sid);
    assert_eq!(
        pending.get(0).unwrap().status,
        SettlementStatus::PartiallySettled
    );
}

#[test]
fn executed_settlement_absent_from_pending_list() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, token_id, sid) = setup(&env, 10_000_000);

    client.partial_settle(&sid, &10_000_000, &token_id);
    let pending = client.get_pending_settlements();
    assert_eq!(pending.len(), 0);
}

#[test]
#[should_panic(expected = "ThresholdNotMet")]
fn partial_settle_without_sufficient_approvals_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let contract_id = env.register_contract(None, TreasuryContract);
    let client = TreasuryContractClient::new(&env, &contract_id);
    client.initialize(&admin, &10); // threshold=10, admin weight=1
    let token_id = env.register_stellar_asset_contract(admin.clone());
    soroban_sdk::token::StellarAssetClient::new(&env, &token_id).mint(&contract_id, &1_000_000);
    let sid = client.propose_settlement(&admin, &merchant, &1_000_000);
    client.partial_settle(&sid, &500_000, &token_id);
}

#[test]
#[should_panic(expected = "InvalidPartialAmount")]
fn partial_settle_exceeding_remainder_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, token_id, sid) = setup(&env, 1_000_000);

    client.partial_settle(&sid, &2_000_000, &token_id);
}
