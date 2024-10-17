use anchor_lang::prelude::*;

#[account]
pub struct UserStake {
    pub is_initialized: bool,
    pub has_claimed_tokens: bool,
    pub invests: u64,
    pub purchased: u64,
    pub bump: u8,
}

impl UserStake {
    pub const LEN: usize = 8 + 16 + 8 + 3;
}
