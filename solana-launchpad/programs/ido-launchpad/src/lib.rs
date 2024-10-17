use anchor_lang::prelude::*;
pub mod error;
pub mod instructions;
pub mod states;
pub mod utils;

use instructions::*;

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("GghUhzqc1sYRKC13vfDzTq5XazGveP4GqxTG52U12qLZ");

#[program]
mod ido_launchpad {
    use super::*;
    pub fn initialize(
        ctx: Context<Initialize>,
        min_invest: u128,
        max_invest: u128,
        token_price: u64,
        start_time: u64,
        end_time: u64,
    ) -> Result<()> {
        instructions::initialize(
            ctx,
            min_invest,
            max_invest,
            token_price,
            start_time,
            end_time,
        )
    }
    //transfer user's token_account to launchpad token account
    pub fn buy_token(ctx: Context<BuyTokens>, amount: u64) -> Result<()> {
        instructions::buy_token(ctx, amount)
    }

    //transfer launchpad's meme token_account to user's token account
    pub fn claim_token(ctx: Context<ClaimTokens>) -> Result<()> {
        instructions::claim_token(ctx)
    }
    //called by admin
    //transfered from launchpad to beneficary for payment token
    pub fn withdraw_payment(ctx: Context<WithdrawFunds>) -> Result<()> {
        instructions::withdraw_payment(ctx)
    }
    pub fn withdraw_meme(ctx: Context<WithdrawMeme>) -> Result<()> {
        instructions::withdraw_meme(ctx)
    }
    pub fn close_launchpad_accounts(ctx: Context<CloseLaunchpadAccounts>) -> Result<()> {
        instructions::close_launchpad_accounts(ctx)
    }
}
