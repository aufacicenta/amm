use crate::*;
use near_sdk::serde_json::json;
use near_sdk::AccountId;

#[ext_contract]
pub trait OracleContractExt {
    fn get_config() -> Promise;
}

pub fn fetch_oracle_config(oracle_contract_id: &str) -> Promise {
    oracle_contract_ext::get_config(&oracle_contract_id, 0, 4_000_000_000_000)
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

impl AMMContract {
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

