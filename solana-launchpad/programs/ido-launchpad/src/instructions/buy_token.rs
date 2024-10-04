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
pub struct BuyTokens<'info> {
    #[account(
      mut,
      constraint = launchpad_state.payment_mint == user_payment_token_account.mint
        && launchpad_state.meme_mint == meme_mint.key()
    )]
    pub launchpad_state: Account<'info, LaunchpadState>,

    #[account(
      init_if_needed,
      seeds=[IDO_LAUNCHPAD_SEED.as_bytes(), launchpad_state.key().as_ref(), signer.key.as_ref()],
      bump,
      payer = signer,
      space = UserStake::LEN
    )]
    pub user_stake: Account<'info, UserStake>,

    #[account(
      mut,
      token::mint = payment_mint
    )]
    pub user_payment_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
      mut,
      seeds = [
          IDO_LAUNCHPAD_SEED.as_bytes(),
          launchpad_state.key().as_ref(),
          payment_mint.key().as_ref()
      ],
      bump
    )]
    pub launchpad_payment_token_account: InterfaceAccount<'info, TokenAccount>,

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

//transfer user's token_account to launchpad token account
pub fn buy_token(ctx: Context<BuyTokens>, amount: u64) -> Result<()> {
    let launchpad_state = &ctx.accounts.launchpad_state;
    let block_timestamp = clock::Clock::get()?.unix_timestamp as u64;
    if launchpad_state.start_time > block_timestamp || launchpad_state.end_time < block_timestamp {
        return err!(ErrCode::InvalidSaleActive);
    }

    if !ctx.accounts.user_stake.is_initialized {
        ctx.accounts.user_stake.is_initialized = true;
        ctx.accounts.user_stake.bump = ctx.bumps.user_stake;
    }
    if amount == 0 {
        return err!(ErrCode::InvalidAmount);
    }
    let cost = amount * launchpad_state.token_price;
    if (cost as u128) < launchpad_state.min_invest || (cost as u128) > launchpad_state.max_invest {
        return err!(ErrCode::InvalidAmount);
    }

    transfer_from_user(
        ctx.accounts.signer.to_account_info(),
        ctx.accounts.user_payment_token_account.to_account_info(),
        ctx.accounts
            .launchpad_payment_token_account
            .to_account_info(),
        ctx.accounts.payment_mint.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        cost,
        ctx.accounts.payment_mint.decimals,
    )?;
    ctx.accounts.user_stake.invests += cost;
    ctx.accounts.user_stake.purchased += amount;
    ctx.accounts.launchpad_state.total_sold += amount;
    emit!(BuyEvent {
        buyer: ctx.accounts.signer.key(),
        payment_mint: ctx.accounts.payment_mint.key(),
        meme_mint: ctx.accounts.meme_mint.key(),
        amount,
    });
    Ok(())
}
