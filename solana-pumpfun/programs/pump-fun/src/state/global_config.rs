
use crate::*;
pub const CONFIG_SEED: &[u8] = b"pumpfun_config";
pub const TOKEN_SEED: &[u8] = b"pumpfun_token";
// pub const TOKEN_MINT_AUTHORITY_SEED: &str = "pumpfun_mint_authority";
pub const BONDING_CURVE_SEED: &[u8] = b"pumpfun_bonding_curve";
pub const WITHDRAWABLE_MIN_SOL_AMOUNT: u64 = 85000000000; //85 SOL

#[account]
#[derive(InitSpace)]
pub struct GlobalConfig {
    pub admin: Pubkey, //also authority to withdraw..
    pub fee_recipient: Pubkey,
    pub fee_rate: u32,
    pub bump: u8,
    pub is_initialized: bool,
}