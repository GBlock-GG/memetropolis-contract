use crate::error::ErrCode;
use crate::states::*;
use crate::utils::*;
use anchor_lang::prelude::*;
use anchor_lang::{accounts::interface_account::InterfaceAccount, solana_program::clock};
use anchor_spl::{
    token::Token,
    token_interface::{Mint, TokenAccount},
};

#[derive(Accounts)]
pub struct WithdrawMeme<'info> {
    #[account(
      mut,
      constraint = launchpad_state.meme_mint == meme_mint.key() &&
                  launchpad_state.admin == signer.key()
    )]
    pub launchpad_state: Account<'info, LaunchpadState>,

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
      mut,
      token::mint = meme_mint
    )]
    pub beneficiary_meme_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
      mint::token_program = token_program,
    )]
    pub meme_mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn withdraw_meme(ctx: Context<WithdrawMeme>) -> Result<()> {
    let block_timestamp = clock::Clock::get()?.unix_timestamp as u64;
    if block_timestamp < ctx.accounts.launchpad_state.end_time {
        return err!(ErrCode::InvalidSaleEnd);
    }
    let withdraw_amount = ctx.accounts.launchpad_meme_token_account.amount
        - (ctx.accounts.launchpad_state.total_sold - ctx.accounts.launchpad_state.claimed_amount);
    transfer_from_launchpad(
        ctx.accounts.launchpad_state.to_account_info(),
        ctx.accounts.launchpad_meme_token_account.to_account_info(),
        ctx.accounts
            .beneficiary_meme_token_account
            .to_account_info(),
        ctx.accounts.meme_mint.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        withdraw_amount,
        ctx.accounts.meme_mint.decimals,
        &[&[
            IDO_LAUNCHPAD_SEED.as_bytes(),
            ctx.accounts.launchpad_state.meme_mint.as_ref(),
            ctx.accounts.launchpad_state.admin.key().as_ref(),
            &[ctx.accounts.launchpad_state.bump],
        ]],
    )?;
    emit!(WithdrawMemeEvent {
        withdrawer: ctx.accounts.signer.key(),
        meme_mint: ctx.accounts.meme_mint.key(),
        amount: withdraw_amount
    });
    Ok(())
}
