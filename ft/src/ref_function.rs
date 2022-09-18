use crate::ref_finance::{ext_ref_finance, REF_CONFIG};
use crate::utils::{
    GAS_FOR_ADD_LIQUIDITY, GAS_FOR_ADD_STABLE_LIQUIDITY, GAS_FOR_FT_TRANSFER_CALL,
    GAS_FOR_MFT_TRANSFER_CALL, GAS_FOR_STAKE, GAS_FOR_STORAGE_DEPOSIT, ONE_NEAR, ONE_YOCTO,
};
use crate::{Contract, ContractExt};
use near_contract_standards::fungible_token::core::ext_ft_core;

use near_sdk::json_types::U128;
use near_sdk::{env, ext_contract, near_bindgen, Balance, Promise, PromiseError};

use std::convert::TryFrom;

// TODO : need to check these whole logic & check if the gas and deposit amount are appropriate
impl Contract {
    pub fn ref_add_stable_liquidity(&self, amount: Balance) -> Promise {
        ext_ft_core::ext(self.usn_token_account_id.clone())
            .with_attached_deposit(ONE_YOCTO)
            .with_static_gas(GAS_FOR_FT_TRANSFER_CALL)
            .ft_transfer_call(
                near_sdk::AccountId::try_from(REF_CONFIG.ref_address.to_string()).unwrap(),
                amount.into(),
                None,
                "".to_string(),
            )
            .then(
                ext_ref_finance::ext(
                    near_sdk::AccountId::try_from(REF_CONFIG.ref_address.to_string()).unwrap(),
                )
                .with_attached_deposit(ONE_NEAR / 100u128)
                .with_static_gas(GAS_FOR_ADD_STABLE_LIQUIDITY)
                .add_stable_liquidity(
                    REF_CONFIG.pool_id,
                    vec![amount.into(), U128::from(0)],
                    U128::from(0),
                ),
            )
            .then(
                ref_callback::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_STAKE)
                    .stake_after_adding_liquidity(),
            )
    }

    pub fn ref_unstake_lp_shares_with_token_amount(&self, amount: Balance) -> Promise {
        ext_ref_finance::ext(
            near_sdk::AccountId::try_from(REF_CONFIG.ref_address.to_string()).unwrap(),
        )
        .get_pool_share_price(REF_CONFIG.pool_id)
        .then(
            ref_callback::ext(
                near_sdk::AccountId::try_from(REF_CONFIG.farm_address.to_string()).unwrap(),
            )
            .with_static_gas(GAS_FOR_ADD_LIQUIDITY)
            .unstake_lp_shares_after_getting_share_price(amount),
        )
        .then(
            ext_ref_finance::ext(
                near_sdk::AccountId::try_from(REF_CONFIG.ref_address.to_string()).unwrap(),
            )
            .with_static_gas(GAS_FOR_ADD_LIQUIDITY)
            .remove_liquidity_by_tokens(
                REF_CONFIG.pool_id,
                vec![amount.into(), U128::from(0u128)],
                amount.into(), // TODO : set strict max_burn_shares with share_price and tolerable slippage
            ),
        )
        .then(
            ext_ref_finance::ext(
                near_sdk::AccountId::try_from(REF_CONFIG.ref_address.to_string()).unwrap(),
            )
            .with_static_gas(GAS_FOR_ADD_LIQUIDITY)
            .withdraw(
                near_sdk::AccountId::try_from(self.usn_token_account_id.to_string()).unwrap(),
                amount.into(),
                Some(false),
            ),
        )
    }

    pub fn ref_claim_and_withdraw_reward(&mut self) -> Promise {
        ext_ref_finance::ext(
            near_sdk::AccountId::try_from(REF_CONFIG.farm_address.to_string()).unwrap(),
        )
        .with_static_gas(GAS_FOR_ADD_LIQUIDITY)
        .claim_reward_by_seed(format!("{}@{}", REF_CONFIG.ref_address, REF_CONFIG.pool_id))
        .then(
            ext_ref_finance::ext(
                near_sdk::AccountId::try_from(REF_CONFIG.farm_address.to_string()).unwrap(),
            )
            .with_static_gas(GAS_FOR_ADD_LIQUIDITY)
            .withdraw_reward(
                near_sdk::AccountId::try_from(REF_CONFIG.reward_token_id.to_string()).unwrap(),
                None,
            ),
        )
    }
}

#[near_bindgen]
impl Contract {
    // It should be called at least once before using this contract
    #[payable]
    pub fn ref_storage_deposit(&mut self) -> Promise {
        ext_ref_finance::ext(
            near_sdk::AccountId::try_from(REF_CONFIG.ref_address.to_string()).unwrap(),
        )
        .with_attached_deposit(ONE_NEAR / 10u128)
        .with_static_gas(GAS_FOR_STORAGE_DEPOSIT)
        .storage_deposit(Some(env::current_account_id()), Some(false))
        .then(
            ext_ref_finance::ext(
                near_sdk::AccountId::try_from(REF_CONFIG.farm_address.to_string()).unwrap(),
            )
            .with_attached_deposit(ONE_NEAR / 10u128)
            .with_static_gas(GAS_FOR_STORAGE_DEPOSIT)
            .storage_deposit(Some(env::current_account_id()), Some(false)),
        )
    }
}

#[ext_contract(ref_callback)]
trait RefCallbacks {
    fn stake_after_adding_liquidity(
        &self,
        #[callback_result] call_result: Result<U128, PromiseError>,
    ) -> Promise;

    fn unstake_lp_shares_after_getting_share_price(
        &self,
        amount: Balance,
        #[callback_result] call_result: Result<U128, PromiseError>,
    ) -> Promise;

    // fn withdraw_reward_after_claim(
    //     &self,
    //     project_account_id: AccountId,
    //     #[callback_result] call_result: Result<U128, PromiseError>,
    // ) -> Promise;
}

#[near_bindgen]
impl RefCallbacks for Contract {
    #[private]
    fn stake_after_adding_liquidity(
        &self,
        #[callback_result] call_result: Result<U128, PromiseError>,
    ) -> Promise {
        let shares: U128 = call_result.unwrap();

        ext_ref_finance::ext(
            near_sdk::AccountId::try_from(REF_CONFIG.ref_address.to_string()).unwrap(),
        )
        .with_attached_deposit(ONE_YOCTO)
        .with_static_gas(GAS_FOR_MFT_TRANSFER_CALL)
        .mft_transfer_call(
            near_sdk::AccountId::try_from(REF_CONFIG.farm_address.to_string()).unwrap(),
            REF_CONFIG.token_id.to_string(),
            shares,
            "\"Free\"".to_string(),
        )
    }

    #[private]
    fn unstake_lp_shares_after_getting_share_price(
        &self,
        amount: Balance,
        #[callback_result] call_result: Result<U128, PromiseError>,
    ) -> Promise {
        let share_price: U128 = call_result.unwrap(); // TODO : check what the unit is, 100000000 ?
        let shares_to_unstake: U128 = U128::from(share_price.0 * amount / 99500000u128);
        // Set 0.5% buffer in order to prevent the situation failing to withdraw
        ext_ref_finance::ext(
            near_sdk::AccountId::try_from(REF_CONFIG.farm_address.to_string()).unwrap(),
        )
        .with_attached_deposit(ONE_NEAR)
        .with_static_gas(GAS_FOR_ADD_LIQUIDITY)
        .unlock_and_withdraw_seed(
            format!("{}@{}", REF_CONFIG.ref_address, REF_CONFIG.pool_id),
            U128::from(0u128),
            shares_to_unstake,
        )
    }
}
