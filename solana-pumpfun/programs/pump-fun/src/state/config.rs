
use crate::*;
pub const CONFIG_SEED: &str = "pumpfun_config";
pub const TOKEN_SEED: &str = "pumpfun_token";
// pub const TOKEN_MINT_AUTHORITY_SEED: &str = "pumpfun_mint_authority";
pub const BONDING_CURVE_SEED: &str = "pumpfun_bonding_curve";
pub const WITHDRAWABLE_MIN_SOL_AMOUNT: u64 = 85000000000; //85 SOL
#[account]
pub struct Config {
    pub initialized: bool,
    pub authority: Pubkey, //admin
    pub fee_recipient: Pubkey,
}

impl Config {
    pub const LEN: usize = 8 + 1 + 32 * 2 + 8 * 2 + 1;
}
