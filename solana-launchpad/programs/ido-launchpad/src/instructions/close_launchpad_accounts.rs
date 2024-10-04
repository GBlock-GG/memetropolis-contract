use anchor_lang::prelude::*;
use anchor_lang::{accounts::interface_account::InterfaceAccount, solana_program::clock};
use anchor_spl::{token::Token, token_interface::Mint};

use crate::error::ErrCode;
use crate::states::*;

#[derive(Accounts)]
pub struct CloseLaunchpadAccounts<'info> {
    #[account(
      mut,
      constraint = launchpad_state.admin == signer.key() &&
                  launchpad_state.meme_mint == meme_mint.key() &&
                  launchpad_state.payment_mint == payment_mint.key(),
      close = signer
    )]
    pub launchpad_state: Account<'info, LaunchpadState>,

    #[account(
      mint::token_program = token_program,
    )]
    pub meme_mint: InterfaceAccount<'info, Mint>,

    #[account(
      mint::token_program = token_program,
    )]
    pub payment_mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn close_launchpad_accounts(ctx: Context<CloseLaunchpadAccounts>) -> Result<()> {
    let block_timestamp = clock::Clock::get()?.unix_timestamp as u64;
    if ctx.accounts.launchpad_state.end_time > block_timestamp {
        return err!(ErrCode::InvalidSaleEnd);
    }
    if ctx.accounts.launchpad_state.claimed_amount < ctx.accounts.launchpad_state.total_sold {
        return err!(ErrCode::RemainedNotClaim);
    }
    Ok(())
}
