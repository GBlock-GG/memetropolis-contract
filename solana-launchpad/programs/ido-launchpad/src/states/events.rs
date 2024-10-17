use anchor_lang::prelude::*;

#[event]
pub struct CreateLaunchpad {
    pub creator: Pubkey,
    pub meme_mint: Pubkey,
    pub payment_mint: Pubkey,
    pub min_invest: u128,
    pub max_invest: u128,
    pub token_price: u64,
    pub start_time: u64,
    pub end_time: u64,
}

#[event]
pub struct BuyEvent {
    pub buyer: Pubkey,
    pub payment_mint: Pubkey,
    pub meme_mint: Pubkey,
    pub amount: u64,
}

#[event]
pub struct ClaimEvent {
    pub claimer: Pubkey,
    pub payment_mint: Pubkey,
    pub meme_mint: Pubkey,
    pub claim_amount: u64,
}

#[event]
pub struct WithdrawMemeEvent {
    pub withdrawer: Pubkey,
    pub meme_mint: Pubkey,
    pub amount: u64,
}

#[event]
pub struct WithdrawPaymentEvent {
    pub withdrawer: Pubkey,
    pub payment_mint: Pubkey,
    pub amount: u64,
}
