#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, token, Address, Env};

#[test]
fn test_1_happy_path_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let station = Address::generate(&env);
    let driver = Address::generate(&env);
    
    // Deploy dummy SAC (Stellar Asset Contract) token to act as PHPC/USDC
    let token_admin = Address::generate(&env);
    let token_contract_id = env.register_stellar_asset_contract(token_admin);
    let token_client = token::Client::new(&env, &token_contract_id);
    
    // Mint 5000 stablecoins to the driver account
    token_client.mint(&driver, &5000);

    // Register our procurement contract 
    let contract_id = env.register_contract(None, GasoLinxContract);
    let client = GasoLinxContractClient::new(&env, &contract_id);

    // Wholesale Price set to 50 stablecoins per liter of gasoline
    client.initialize(&station, &token_contract_id, &50);

    // Driver spends 2500 stablecoins to lock in fuel allocation volumes early
    client.pre_purchase_fuel(&driver, &2500);

    // 2500 / 50 = 50 liters allocated
    assert_eq!(client.get_driver_liters(&driver), 50);
    assert_eq!(token_client.balance(&contract_id), 2500);

    // Station executes batch delivery payout allocation order
    let paid_out = client.dispatch_wholesale_payout();
    assert_eq!(paid_out, 2500);
    assert_eq!(token_client.balance(&station), 2500);
}

#[test]
#[should_panic(expected = "Deposit amount insufficient to purchase at least one liter")]
fn test_2_edge_case_insufficient_deposit() {
    let env = Env::default();
    env.mock_all_auths();

    let station = Address::generate(&env);
    let driver = Address::generate(&env);
    let token_contract_id = env.register_stellar_asset_contract(Address::generate(&env));

    let contract_id = env.register_contract(None, GasoLinxContract);
    let client = GasoLinxContractClient::new(&env, &contract_id);

    client.initialize(&station, &token_contract_id, &50);
    
    // Deposit of only 10 stablecoins cannot buy 1 liter costing 50 tokens
    client.pre_purchase_fuel(&driver, &10);
}

#[test]
fn test_3_state_verification() {
    let env = Env::default();
    env.mock_all_auths();

    let station = Address::generate(&env);
    let driver_1 = Address::generate(&env);
    let driver_2 = Address::generate(&env);
    let token_contract_id = env.register_stellar_asset_contract(Address::generate(&env));
    let token_client = token::Client::new(&env, &token_contract_id);

    token_client.mint(&driver_1, &1000);
    token_client.mint(&driver_2, &1000);

    let contract_id = env.register_contract(None, GasoLinxContract);
    let client = GasoLinxContractClient::new(&env, &contract_id);

    client.initialize(&station, &token_contract_id, &50);

    client.pre_purchase_fuel(&driver_1, &500); // 10 liters
    client.pre_purchase_fuel(&driver_2, &1000); // 20 liters

    assert_eq!(client.get_driver_liters(&driver_1), 10);
    assert_eq!(client.get_driver_liters(&driver_2), 20);
}

#[test]
#[should_panic(expected = "Contract already initialized")]
fn test_4_edge_case_duplicate_initialization() {
    let env = Env::default();
    let station = Address::generate(&env);
    let token_contract_id = env.register_stellar_asset_contract(Address::generate(&env));

    let contract_id = env.register_contract(None, GasoLinxContract);
    let client = GasoLinxContractClient::new(&env, &contract_id);

    client.initialize(&station, &token_contract_id, &50);
    client.initialize(&station, &token_contract_id, &50); // Re-triggering initialization must fail
}

#[test]
#[should_panic(expected = "No liquidity pooled to pay out")]
fn test_5_edge_case_empty_payout_rejection() {
    let env = Env::default();
    env.mock_all_auths();

    let station = Address::generate(&env);
    let token_contract_id = env.register_stellar_asset_contract(Address::generate(&env));

    let contract_id = env.register_contract(None, GasoLinxContract);
    let client = GasoLinxContractClient::new(&env, &contract_id);

    client.initialize(&station, &token_contract_id, &50);
    client.dispatch_wholesale_payout(); // Panic: no drivers have deposited yet
}