use crate::utils::*;
use near_sdk::json_types::{U128};
use near_sdk_sim::{to_yocto};
use amm::types::{Outcome, AnswerType};

#[test]
fn test_valid_market_resolution_oracle() {
    let test_utils = TestUtils::init(carol());
    
    // variables
    let market_id = 0;
    
    // create market and data request
    test_utils.alice.create_market(2, Some(U128(0)));
    test_utils.alice.create_data_request(market_id);
    
    // check if the dr exists
    let dr_exist = test_utils.alice.dr_exists(0);
    assert!(dr_exist, "something went wrong during dr creation");
    
    let target_price = to_yocto("5") / 10;
    let seed_amount = to_yocto("100");
    let weights = Some(calc_weights_from_price(vec![target_price, target_price]));
    
    test_utils.alice.add_liquidity(market_id, seed_amount, weights);
    
    // stake on and finalize data request on oracle
    let outcome_to_stake = Outcome::Answer(AnswerType::String(empty_string()));
    test_utils.carol.stake(0, outcome_to_stake.clone(), 200);
    println!("Bonded outcome: {:?}", test_utils.alice.get_latest_request().unwrap().resolution_windows[0].bonded_outcome);
    
    // resolute market on amm
    test_utils.alice.finalize(0);
    test_utils.carol.resolute_market(market_id, None);
}

#[test]
fn test_valid_market_payout() {
    
    let test_utils = TestUtils::init(carol());
    
    // variables
    let market_id = 0;
    
    test_utils.alice.create_market(2, Some(U128(0)));
    let target_price = to_yocto("5") / 10;
    let seed_amount = to_yocto("100");
    let buy_amount = to_yocto("1");
    let weights = Some(calc_weights_from_price(vec![target_price, target_price]));
    
    test_utils.alice.add_liquidity(market_id, seed_amount, weights);
    
    let payout_num = vec![U128(0), U128(to_yocto("1"))];
    
    test_utils.bob.buy(market_id, buy_amount, 0, 0);
    test_utils.bob.buy(market_id, buy_amount, 1, 0);
    test_utils.bob.buy(market_id, buy_amount, 0, 0);
    test_utils.bob.buy(market_id, buy_amount, 1, 0);
    
    test_utils.carol.resolute_market(market_id, Some(payout_num));
    
    let pre_claim_balance = test_utils.bob.get_token_balance(None);
    
    assert_eq!(pre_claim_balance, init_balance() / 2 - buy_amount * 4, "unexpected balance");
    
    test_utils.bob.claim_earnings(market_id);
    
    let claimer_balance: u128 = test_utils.bob.get_token_balance(None);
    let expected_claimer_balance = 500019603038518995487419933_u128;
    assert_eq!(claimer_balance, expected_claimer_balance, "unexpected payout");
}

#[test]
fn test_invalid_market_payout() {
    let test_utils = TestUtils::init(carol());
    
    // variables
    let market_id = 0;
    let creation_bond = 100;
    let alice_init_balance: u128 = test_utils.alice.get_token_balance(None);
    let bob_init_balance: u128 = test_utils.bob.get_token_balance(None);
    
    let expected_alice_final_balance = alice_init_balance;
    let expected_bob_final_balance = bob_init_balance;
    let expected_amm_final_balance = 0;
    
    let target_price = to_yocto("5") / 10;
    let seed_amount = to_yocto("100");
    let buy_amount = to_yocto("1");
    let weights = Some(calc_weights_from_price(vec![target_price, target_price]));
    
    test_utils.alice.create_market(2, Some(U128(0)));
    test_utils.alice.create_data_request(market_id);
    test_utils.alice.add_liquidity(market_id, seed_amount, weights);
    
    test_utils.bob.buy(market_id, buy_amount, 0, 0);
    test_utils.bob.buy(market_id, buy_amount, 1, 0);
    test_utils.bob.buy(market_id, buy_amount, 0, 0);
    test_utils.bob.buy(market_id, buy_amount, 1, 0);

    test_utils.bob.sell(market_id, buy_amount, 0, to_yocto("100"));

    test_utils.alice.exit_liquidity(market_id, seed_amount);

    let outcome_to_stake = Outcome::Answer(AnswerType::String(empty_string()));
    let amount_to_stake = 200;
    test_utils.carol.stake(0, outcome_to_stake.clone(), amount_to_stake);
    test_utils.alice.finalize(0);

    test_utils.carol.resolute_market(market_id, None);

    test_utils.bob.claim_earnings(market_id);
    test_utils.alice.claim_earnings(market_id);
    
    let alice_final_balance = test_utils.alice.get_token_balance(None);
    let bob_final_balance = test_utils.bob.get_token_balance(None);
    let amm_final_balance = test_utils.bob.get_token_balance(Some(AMM_CONTRACT_ID.to_string()));

    // Assert balances
    assert_eq!(alice_final_balance, expected_alice_final_balance - creation_bond + 100);
    assert_eq!(bob_final_balance, expected_bob_final_balance);
    assert_eq!(amm_final_balance, expected_amm_final_balance);
}

#[test]
fn payout_lp_no_exit() {
    let test_utils = TestUtils::init(carol());
    
    // variables
    let creation_bond = 100;
    let market_id = 0;
    let alice_init_balance: u128 = test_utils.alice.get_token_balance(None);
    let bob_init_balance: u128 = test_utils.bob.get_token_balance(None);
    
    let expected_alice_final_balance = alice_init_balance;
    let expected_bob_final_balance = bob_init_balance;
    let expected_amm_final_balance = 0;
    
    let target_price = to_yocto("5") / 10;
    let seed_amount = to_yocto("100");
    let buy_amount = to_yocto("1");
    let weights = Some(calc_weights_from_price(vec![target_price, target_price]));

    test_utils.alice.create_market(2, Some(U128(0)));
    test_utils.alice.create_data_request(market_id);
    test_utils.alice.add_liquidity(market_id, seed_amount, weights);

    test_utils.bob.buy(market_id, buy_amount, 0, 0);
    test_utils.bob.buy(market_id, buy_amount, 1, 0);
    test_utils.bob.buy(market_id, buy_amount, 0, 0);
    test_utils.bob.buy(market_id, buy_amount, 1, 0);

    test_utils.bob.sell(market_id, buy_amount, 0, to_yocto("100"));

    let outcome_to_stake = Outcome::Answer(AnswerType::String(empty_string()));
    let amount_to_stake = 200;
    test_utils.carol.stake(0, outcome_to_stake.clone(), amount_to_stake);

    test_utils.alice.finalize(0);
    test_utils.carol.resolute_market(market_id, None);

    test_utils.bob.claim_earnings(market_id);
    test_utils.alice.claim_earnings(market_id);
    
    let alice_final_balance = test_utils.alice.get_token_balance(None);
    let bob_final_balance = test_utils.bob.get_token_balance(None);
    let amm_final_balance = test_utils.bob.get_token_balance(Some(AMM_CONTRACT_ID.to_string()));

    // Assert balances
    assert_eq!(alice_final_balance, expected_alice_final_balance - creation_bond + 100);
    assert_eq!(bob_final_balance, expected_bob_final_balance);
    assert_eq!(amm_final_balance, expected_amm_final_balance);
    
}