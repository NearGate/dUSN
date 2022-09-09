use near_sdk::{Balance, Gas};

const ONE_E24: u128 = 1_000_000_000_000_000_000_000_000;
const NEAR: u128 = ONE_E24;
pub const ONE_NEAR: u128 = NEAR;
pub const MILLI_NEAR: u128 = 1_000_000_000_000_000_000;
pub const ONE_YOCTO: Balance = 1;

pub const GAS_FOR_STORAGE_DEPOSIT: Gas = Gas(5_000_000_000_000);
pub const GAS_FOR_FT_TRANSFER_CALL: Gas = Gas(8_000_000_000_000);
pub const GAS_FOR_ADD_LIQUIDITY: Gas = Gas(17_000_000_000_000);
pub const GAS_FOR_MFT_TRANSFER_CALL: Gas = Gas(18_000_000_000_000);
pub const GAS_FOR_ADD_STABLE_LIQUIDITY: Gas = Gas(6_000_000_000_000);
pub const GAS_FOR_STAKE: Gas = Gas(23_000_000_000_000);
