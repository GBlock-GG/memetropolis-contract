
use crate::*;
pub const CONFIG_SEED: &str = "pumpfun_config";
pub const TOKEN_SEED: &str = "pumpfun_token";
// pub const TOKEN_MINT_AUTHORITY_SEED: &str = "pumpfun_mint_authority";
pub const BONDING_CURVE_SEED: &str = "pumpfun_bonding_curve";
pub const WITHDRAWABLE_MIN_SOL_AMOUNT: u64 = 85000000000; //85 SOL

#[account]
#[derive(InitSpace)]
pub struct GlobalConfig {
    pub admin: Pubkey, //admin  to withdraw..
    pub fee_recipient: Pubkey,
    pub fee_rate: u32,
    pub bump: u8
}