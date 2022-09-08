use near_sdk::{Balance, Gas};

const ONE_E24: u128 = 1_000_000_000_000_000_000_000_000;
const NEAR: u128 = ONE_E24;
pub const ONE_NEAR: u128 = NEAR;
pub const ONE_YOCTO: Balance = 1;

pub const GAS_FOR_ADD_LIQUIDITY: Gas = Gas(17_000_000_000_000);
