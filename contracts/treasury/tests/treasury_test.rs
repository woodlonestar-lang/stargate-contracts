use soroban_sdk::{testutils::Address as _, Address, Env};
use treasury::{SettlementStatus, TreasuryContract, TreasuryContractClient};

fn setup() -> (Env, Address, Address, TreasuryContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let backup = Address::generate(&env);
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
}

#[test]
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
