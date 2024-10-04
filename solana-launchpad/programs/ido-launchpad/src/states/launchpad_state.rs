use anchor_lang::prelude::*;

pub const IDO_LAUNCHPAD_SEED: &str = "ido_launchpad";

#[account]
pub struct LaunchpadState {
    pub admin: Pubkey,
    pub meme_mint: Pubkey,
    pub payment_mint: Pubkey,
    pub start_time: u64,
    pub end_time: u64,
    pub max_invest: u128,
    pub min_invest: u128,
    pub token_price: u64,
    pub total_sold: u64,
    pub claimed_amount: u64,
    pub bump: u8,
}

impl LaunchpadState {
    pub const LEN: usize = 8 + 32 * 3 + 8 * 2 + 16 * 5 + 1;
}
