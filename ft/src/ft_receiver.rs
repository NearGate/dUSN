use crate::utils::ONE_YOCTO;
use crate::{Contract, ContractExt};

use near_contract_standards::fungible_token::core::ext_ft_core;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::json_types::U128;
use near_sdk::{
    env, ext_contract, is_promise_success, log, near_bindgen, AccountId, Balance, Gas, Promise,
    PromiseOrValue,
};

const BASE_GAS: Gas = Gas(5_000_000_000_000);
pub(crate) const FT_TRANSFER_GAS: Gas = BASE_GAS;
pub(crate) const AFTER_FT_TRANSFER_GAS: Gas = BASE_GAS;

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let mut promise: PromiseOrValue<U128> = PromiseOrValue::Value(0.into());
        let token_account_id = env::predecessor_account_id();
        match msg.as_str() {
            "deposit" => {
                assert!(token_account_id == self.usn_token_account_id, "Only USN can be deposited");
                // TODO : check if 10 is proper amount as a minimal threshold
                assert!(
                    amount.0 >= 10_000_000_000_000_000_000u128,
                    "USN Deposit amount should be more than 10"
                );

                // need to calculate the price of dUSN
                // TODO : check if it is okay to add 1/10^18 dUSN for avoiding zero division error
                let dusn_total_supply: Balance = self.token.total_supply + 1u128;
                // TODO : check overflow when multiplication
                let dusn_return_amount: Balance = amount.0 * dusn_total_supply / self.usn_reserve;

                // send dUSN to the sender
                // storage_deposit needs to be called on Web side in advance
                // TODO : check if refund works well when panic happens
                self.token.internal_deposit(&sender_id, dusn_return_amount);
                // This emission seems unnecessary
                near_contract_standards::fungible_token::events::FtMint {
                    owner_id: &env::current_account_id(),
                    amount: &U128::from(dusn_return_amount),
                    memo: Some("dUSN is minted"),
                }
                .emit();
            }
            "redeem" => {
                assert!(
                    token_account_id == env::current_account_id(),
                    "Only dUSN can be redeemed to USN"
                );

                // need to calculate the price of USN
                // TODO : check if it is okay to add 1/10^18 dUSN for avoiding zero division error
                let dusn_total_supply: Balance = self.token.total_supply + 1u128;
                let usn_return_amount: Balance = amount.0 * self.usn_reserve / dusn_total_supply;
                // TODO : check if 10 USN is proper amount as a minimal threshold
                assert!(
                    usn_return_amount >= 10_000_000_000_000_000_000u128,
                    "dUSN Deposit amount should be more than {}, which is equivalent to 10 USN",
                    10u128 * dusn_total_supply / self.usn_reserve
                );

                // withdraw USN and send it to the sender
                promise = PromiseOrValue::Promise(self.withdraw_and_transfer_usn(
                    sender_id,
                    usn_return_amount,
                    amount.0,
                ));
            }
            _ => {}
        }
        if token_account_id == self.usn_token_account_id {
            // It should be called even without msg.(e.g. getting rewards from Ref finance)
            self.usn_reserve += amount.0;
            // 너무 받을 때 마다하면 이자에 비해 손해를 볼 거같은데
            promise = PromiseOrValue::Promise(self.deposit_usn(amount.0));
        } else if token_account_id == env::current_account_id() {
            // burn dUSN
            self.token.internal_withdraw(&env::current_account_id(), amount.into());
            near_contract_standards::fungible_token::events::FtBurn {
                owner_id: &env::current_account_id(),
                amount: &amount,
                memo: Some("dUSN is burned"),
            }
            .emit();
        }
        promise
    }
}

#[ext_contract(ext_self)]
trait SelfCallbacks {
    fn after_usn_transfer(
        &mut self,
        account_id: AccountId,
        usn_return_amount: Balance,
        dusn_received_amount: Balance,
    ) -> bool;
}

#[near_bindgen]
impl SelfCallbacks for Contract {
    #[private]
    fn after_usn_transfer(
        &mut self,
        account_id: AccountId,
        usn_return_amount: Balance,
        dusn_received_amount: Balance,
    ) -> bool {
        let promise_success = is_promise_success();
        if is_promise_success() {
            self.usn_reserve -= usn_return_amount;
        } else {
            log!(
                "Failed to send {} USN to {}, refunded {} dUSN",
                usn_return_amount,
                account_id,
                dusn_received_amount,
            );
            self.token.internal_deposit(&account_id, dusn_received_amount);
            // let config: Config = self.get_config();
            // if config.in_token_account_id == token_account_id {
            //     let mut account = self.internal_unwrap_account(&account_id);
            //     account.internal_token_deposit(&token_account_id, amount.0);
            // }
        }
        promise_success
    }
}

impl Contract {
    // For now, use Ref finance for getting USN yield
    pub fn deposit_usn(&mut self, amount: Balance) -> Promise {
        self.ref_add_stable_liquidity(amount)
    }

    pub fn withdraw_and_transfer_usn(
        &mut self,
        sender_id: AccountId,
        amount: Balance,
        dusn_received_amount: Balance,
    ) -> Promise {
        self.ref_unstake_lp_shares_with_token_amount(amount).then(
            ext_ft_core::ext(self.usn_token_account_id.clone())
                .with_attached_deposit(ONE_YOCTO)
                .with_static_gas(FT_TRANSFER_GAS)
                .ft_transfer(sender_id.clone(), amount.into(), None)
                .then(
                    ext_self::ext(env::current_account_id())
                        .with_static_gas(AFTER_FT_TRANSFER_GAS)
                        .after_usn_transfer(sender_id, amount, dusn_received_amount),
                ),
        )
    }
}
