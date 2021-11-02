use crate::*;
use near_sdk::serde_json::json;
use near_sdk::{AccountId, PromiseResult};

#[derive(Serialize, Deserialize)]
pub struct OracleConfig {
    pub payment_token: AccountId, // bond token from the oracle config
    pub validity_bond: U128 // validity bond amount
}

#[ext_contract]
pub trait OracleContractExt {
    fn get_config() -> Promise;
}

#[ext_contract(ext_self)]
trait ProtocolResolver {
    fn proceed_data_request_creation(
        &mut self, 
        sender: AccountId, 
        payment_token: AccountId, 
        bond_in: WrappedBalance, 
        market_id: U64 
    ) -> Promise;
}

#[derive(Deserialize, Serialize, BorshDeserialize, BorshSerialize)]
pub enum DataRequestDataType {
    Number(U128),
    String,
}

pub struct NewDataRequestArgs {
    pub sources: Option<Vec<Source>>,
    pub tags: Option<Vec<String>>,
    pub description: Option<String>,
    pub outcomes: Option<Vec<String>>,
    pub challenge_period: WrappedTimestamp,
    pub data_type: DataRequestDataType,
    pub creator: AccountId,
}

const GAS_BASE_CREATE_REQUEST: Gas = 50_000_000_000_000;

#[near_bindgen]
impl AMMContract {
    pub fn proceed_data_request_creation(&mut self, sender: AccountId, payment_token: AccountId, bond_in: WrappedBalance, market_id: U64) -> PromiseOrValue<U128> {
        assert_self();
        assert_prev_promise_successful();

        
        // Maybe we don't need to check. We could also assume that
        // the oracle promise handles the validation..
        let oracle_config = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(value) => {
                match serde_json::from_slice::<OracleConfig>(&value) {
                    Ok(value) => value,
                    Err(_e) => panic!("ERR_INVALID_ORACLE_CONFIG"),
                }
            },
            PromiseResult::Failed => panic!("ERR_FAILED_ORACLE_CONFIG_FETCH"),
        };
        
        let validity_bond: u128 = oracle_config.validity_bond.into();
        let bond_in: u128 = bond_in.into();
        let mut market = self.get_market_expect(market_id);
    
        assert_eq!(oracle_config.payment_token, payment_token, "ERR_INVALID_PAYMENT_TOKEN");
        assert!(validity_bond <= bond_in, "ERR_NOT_ENOUGH_BOND: FOUND {}, NEED {}", bond_in, validity_bond);
    
        let outcomes: Option<Vec<String>> = if market.is_scalar {
            None
        } else {
            Some(flatten_outcome_tags(&market.outcome_tags))
        };
    
        let data_type: DataRequestDataType = if market.is_scalar {
            DataRequestDataType::Number(market.scalar_multiplier.unwrap())
        } else {
            DataRequestDataType::String
        };
    
        let remaining_bond: u128 = bond_in - validity_bond;
        let create_promise = self.create_data_request(&payment_token.clone(), validity_bond, NewDataRequestArgs {
            description: Some(format!("{} - {}", market.description, market.extra_info)),
            outcomes,
            tags: Some(vec![market_id.0.to_string()]),
            sources: Some(market.sources),
            challenge_period: market.challenge_period,
            data_type,
            creator: sender.to_string(),
        });

        // update market with payment token, creator, and payment to return
        market.payment_token = Some(payment_token.clone());
        market.dr_creator = Some(sender.to_string());
        market.validity_bond = Some(validity_bond);
        
        // Refund the remaining tokens
        if remaining_bond > 0 {
            PromiseOrValue::Promise(create_promise
                .then(fungible_token::fungible_token_transfer(&payment_token, sender, remaining_bond)))
        } else {
            PromiseOrValue::Value(U128(0))
        }
    }

}

impl AMMContract {
    // called by ft_on_transfer
    pub fn ft_create_data_request_callback(
        &mut self, 
        sender: &AccountId, 
        bond_in: Balance, 
        payload: CreateDataRequestArgs
    ) -> Promise {
        self.assert_unpaused();

        // only allow data request to be created after resolution_time
        let market = self.get_market_expect(payload.market_id);
        assert!(market.resolution_time <= env::block_timestamp(), "ERR_RESOLUTION_TIME_NOT_REACHED");

        oracle_contract_ext::get_config(&self.oracle, 0, 4_000_000_000_000)
            .then(
                ext_self::proceed_data_request_creation(
                sender.to_string(), 
                env::predecessor_account_id(), 
                U128(bond_in), 
                payload.market_id,
                &env::current_account_id(), 
                0, 
                150_000_000_000_000
            )
        )
    }
    
    // called in proceed_data_request_creation
    pub fn create_data_request(&self, payment_token: &AccountId, amount: Balance, request_args: NewDataRequestArgs) -> Promise {
        // Should do a fungible token transfer to the oracle
        fungible_token::fungible_token_transfer_call(
            payment_token, 
            self.oracle.to_string(), 
            amount,
            json!({
                "NewDataRequest": {
                    // 12 hours in nano seconds
                    "challenge_period": request_args.challenge_period,
                    "outcomes": request_args.outcomes,
                    "sources": request_args.sources,
                    "description": request_args.description,
                    "tags": request_args.tags,
                    "data_type": request_args.data_type,
                    "creator": request_args.creator,
                },
            }).to_string(),
            Some(GAS_BASE_CREATE_REQUEST),
        )
    }
}

