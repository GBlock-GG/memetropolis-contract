use anchor_lang::prelude::*;
use anchor_lang::{accounts::interface_account::InterfaceAccount, solana_program::clock};
use anchor_spl::{
    token::Token,
    token_interface::{Mint, TokenAccount},
};

use crate::error::ErrCode;
use crate::states::*;
use crate::utils::*;

#[derive(Accounts)]
pub struct ClaimTokens<'info> {
    #[account(
      mut,
      constraint = launchpad_state.payment_mint == payment_mint.key() &&
                  launchpad_state.meme_mint == user_meme_token_account.mint &&
                  launchpad_state.meme_mint == meme_mint.key()
    )]
    pub launchpad_state: Account<'info, LaunchpadState>,

    #[account(
      mut,
      seeds=[IDO_LAUNCHPAD_SEED.as_bytes(), launchpad_state.key().as_ref(), signer.key.as_ref()],
      bump,
      close = signer
    )]
    pub user_stake: Account<'info, UserStake>,

    #[account(
      mut,
      token::mint = meme_mint
    )]
    pub user_meme_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
      mut,
      token::mint = meme_mint,
      seeds = [
        IDO_LAUNCHPAD_SEED.as_bytes(),
        launchpad_state.key().as_ref(),
        meme_mint.key().as_ref()
      ],
      bump
    )]
    pub launchpad_meme_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
      mint::token_program = token_program,
    )]
    pub payment_mint: InterfaceAccount<'info, Mint>,

    #[account(
    mint::token_program = token_program,
  )]
    pub meme_mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

//transfer launchpad's meme token_account to user's token account
pub fn claim_token(ctx: Context<ClaimTokens>) -> Result<()> {
    let block_timestamp = clock::Clock::get()?.unix_timestamp as u64;
    if block_timestamp <= ctx.accounts.launchpad_state.end_time {
        return err!(ErrCode::InvalidSaleEnd);
    }
    if ctx.accounts.user_stake.has_claimed_tokens {
        return err!(ErrCode::InvalidClaim);
    }

    let claim_amount: u64 = ctx.accounts.user_stake.purchased;

    transfer_from_launchpad(
        ctx.accounts.launchpad_state.to_account_info(),
        ctx.accounts.launchpad_meme_token_account.to_account_info(),
        ctx.accounts.user_meme_token_account.to_account_info(),
        ctx.accounts.meme_mint.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        claim_amount,
        ctx.accounts.meme_mint.decimals,
        &[&[
            IDO_LAUNCHPAD_SEED.as_bytes(),
            ctx.accounts.launchpad_state.meme_mint.as_ref(),
            ctx.accounts.launchpad_state.admin.key().as_ref(),
            &[ctx.accounts.launchpad_state.bump],
        ]],
    )?;
    ctx.accounts.launchpad_state.claimed_amount += claim_amount;
    emit!(ClaimEvent {
        claimer: ctx.accounts.signer.key(),
        claim_amount,
        payment_mint: ctx.accounts.payment_mint.key(),
        meme_mint: ctx.accounts.meme_mint.key(),
    });
    Ok(())
}
