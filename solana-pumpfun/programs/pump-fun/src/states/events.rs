use anchor_lang::prelude::*;

#[event]
pub struct SwapEvent{
    pub pool_id: Pubkey,
}