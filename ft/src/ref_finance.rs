use crate::*;

type SeedId = String;

#[ext_contract(ext_ref_finance)]
trait RefFinance {
    #[payable]
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance;

    fn get_pool_shares(&self, pool_id: u64, account_id: AccountId) -> U128;

    #[payable]
    fn add_stable_liquidity(&mut self, pool_id: u64, amounts: Vec<U128>, min_shares: U128) -> U128;

    fn mft_balance_of(&self, token_id: String, account_id: AccountId) -> U128;

    #[payable]
    fn mft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: String,
        amount: U128,
        msg: String,
    ) -> U128;

    #[payable]
    fn remove_liquidity(&mut self, pool_id: u64, shares: U128, min_amounts: Vec<U128>)
        -> Vec<U128>;

    #[payable]
    fn withdraw(&mut self, token_id: AccountId, amount: U128, unregister: Option<bool>);

    //// For claiming the interest
    // claim reward with seed, seed_id example : v2.ref-finance.near@3020
    #[payable]
    fn claim_reward_by_seed(&mut self, seed_id: SeedId);

    // get accumulated rewards which are claimed but not withdrawn
    // farmer_id : this contract address, token_id : "usn"
    // TODO : params' origin data type is String in the Ref contract, need to test whether if working well
    fn get_farmer_reward(&self, farmer_id: AccountId, token_id: AccountId) -> U128;

    #[payable]
    fn withdraw_reward(&mut self, token_id: AccountId, amount: Option<U128>);

    fn get_pool_share_price(&self, pool_id: u64) -> U128;

    #[payable]
    fn unlock_and_withdraw_seed(
        &mut self,
        seed_id: String,
        unlock_amount: U128,
        withdraw_amount: U128,
    );

    // return burned shares
    #[payable]
    fn remove_liquidity_by_tokens(
        &mut self,
        pool_id: u64,
        amounts: Vec<U128>,
        max_burn_shares: U128,
    ) -> U128;
}

pub struct RefConfig {
    pub ref_address: &'static str,
    pub farm_address: &'static str,
    pub pool_id: u64,
    pub token_id: &'static str,
    pub reward_token_id: &'static str,
}

pub(crate) const REF_CONFIG: RefConfig = if cfg!(feature = "mainnet") {
    RefConfig {
        ref_address: "v2.ref-finance.near",
        farm_address: "boostfarm.ref-labs.near",
        pool_id: 3020,
        token_id: ":3020",
        reward_token_id: "usn",
    }
} else if cfg!(feature = "testnet") {
    RefConfig {
        ref_address: "ref-finance-101.testnet",
        farm_address: "boostfarm.ref-finance.testnet",
        pool_id: 494,
        token_id: ":494",
        reward_token_id: "usdc.fakes.testnet",
    }
} else {
    RefConfig {
        ref_address: "ref.test.near",
        farm_address: "boostfarm-ref.test.near",
        pool_id: 3020,
        token_id: ":3020",
        reward_token_id: "usn",
    }
};
