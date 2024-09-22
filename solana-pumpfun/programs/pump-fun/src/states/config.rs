use anchor_lang::prelude::*;

pub const CONFIG_SEED: &str = "pumpfun_config";

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