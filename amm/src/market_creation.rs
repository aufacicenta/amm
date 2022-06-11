use crate::*;

#[near_bindgen]
impl AMMContract {
    /**
     * @notice allows users to create new markets, can only be called internally
     * This function assumes the market data has been validated beforehand (ft_create_market_callback)
     * @param description is a detailed description of the market
     * @param extra_info extra information on how the market should be resoluted
     * @param outcomes the number of possible outcomes for the market
     * @param outcome_tags is a list of outcomes where the index is the `outcome_id`
     * @param categories is a list of categories to filter the market by
     * @param end_time when the trading should stop
     * @param resolution_time when the market can be resolved
     * @param collateral_token_id the `account_id` of the whitelisted token that is used as collateral for trading
     * @param swap_fee the fee that's taken from every swap and paid out to LPs
     * @param is_scalar if the market is a scalar market (range)
     * @returns wrapped `market_id` 
     */
    pub fn create_market(&mut self, payload: &CreateMarketArgs) -> U64 {
        self.assert_unpaused();
        let swap_fee: u128 = payload.swap_fee.into();
        let market_id = self.markets.len();
        let token_decimals = self.collateral_whitelist.0.get(&payload.collateral_token_id);
        let end_time: u64 = payload.end_time.into();
        let resolution_time: u64 = payload.resolution_time.into();

        assert!(token_decimals.is_some(), "ERR_INVALID_COLLATERAL");
        assert!(payload.outcome_tags.len() as u16 == payload.outcomes, "ERR_INVALID_TAG_LENGTH");
        assert!(end_time > ns_to_ms(env::block_timestamp()), "ERR_INVALID_END_TIME");
        assert!(resolution_time >= end_time, "ERR_INVALID_RESOLUTION_TIME");

        let pool = pool_factory::new_pool(
            market_id,
            payload.outcomes,
            payload.collateral_token_id.to_string(),
            token_decimals.unwrap(),
            swap_fee
        );

        logger::log_pool(&pool);

        let mut market = Market {
            end_time: payload.end_time.into(),
            resolution_time: payload.resolution_time.into(),
            pool,
            payout_numerator: None,
            finalized: false,
            // Disable this market until the oracle request has been made
            enabled: false,
            is_scalar: payload.is_scalar,
            outcome_tags: payload.outcome_tags.clone(),
        };

        logger::log_create_market(&market, &payload.description, &payload.extra_info, &payload.categories);
        logger::log_market_status(&market);

        // Enable Market
        market.enabled = true;
        logger::log_market_status(&market);

        self.markets.push(&market);
        market_id.into()
    }
}
