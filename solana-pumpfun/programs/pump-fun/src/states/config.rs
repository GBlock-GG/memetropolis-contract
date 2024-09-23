use anchor_lang::prelude::*;

pub const CONFIG_SEED: &str = "pumpfun_config";
pub const TOKEN_SEED: &str = "pumpfun_token";
pub const TOKEN_MINT_AUTHORITY_SEED: &str = "pumpfun_mint_authority";
pub const BONDING_CURVE_SEED: &str = "pumpfun_bonding_curve";

#[account]
pub struct Config {
    pub initialized: bool,
    pub authority: Pubkey, //admin
    pub fee_recipient: Pubkey,
    pub max_supply: u64,  // 1000000
    pub init_supply: u64, //  200000
    pub default_decimals: u8, //6
}

impl Config {
    pub const LEN: usize = 8 + 1 + 32 * 2 + 8 * 2 + 1;
}