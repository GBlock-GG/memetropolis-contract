use anchor_lang::prelude::*;
use anchor_lang::{accounts::interface_account::InterfaceAccount, solana_program::clock};
use anchor_spl::{token::Token, token_interface::Mint};

use crate::error::ErrCode;
use crate::states::*;
use crate::utils::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
      init,
      seeds=[IDO_LAUNCHPAD_SEED.as_bytes(), meme_mint.key().as_ref(), signer.key.as_ref()],
      bump,
      payer = signer,
      space = LaunchpadState::LEN)
    ]
    pub launchpad_state: Account<'info, LaunchpadState>,

    /// CHECK
    #[account(
      mut,
      seeds = [
        IDO_LAUNCHPAD_SEED.as_bytes(),
        launchpad_state.key().as_ref(),
        meme_mint.key().as_ref()
      ],
      bump
    )]
    pub meme_token_account: UncheckedAccount<'info>,

    /// CHECK
    #[account(
      mut,
      seeds = [
          IDO_LAUNCHPAD_SEED.as_bytes(),
          launchpad_state.key().as_ref(),
          payment_mint.key().as_ref()
      ],
      bump
    )]
    pub payment_token_account: UncheckedAccount<'info>,

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

pub fn initialize(
    ctx: Context<Initialize>,
    min_invest: u128,
    max_invest: u128,
    token_price: u64,
    start_time: u64,
    end_time: u64,
) -> Result<()> {
    let block_timestamp = clock::Clock::get()?.unix_timestamp as u64;
    if start_time >= end_time || start_time <= block_timestamp || end_time <= block_timestamp {
        return err!(ErrCode::InvalidTime);
    }
    if token_price == 0 {
        return err!(ErrCode::InvalidPrice);
    }
    //create meme_token_account
    create_token_account(
        &ctx.accounts.launchpad_state.to_account_info(),
        &ctx.accounts.signer.to_account_info(),
        &ctx.accounts.meme_token_account.to_account_info(),
        &ctx.accounts.meme_mint.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
        &ctx.accounts.token_program.to_account_info(),
        &[&[
            IDO_LAUNCHPAD_SEED.as_bytes(),
            ctx.accounts.launchpad_state.key().as_ref(),
            ctx.accounts.meme_mint.key().as_ref(),
            &[ctx.bumps.meme_token_account][..],
        ][..]],
    )?;
    //create payment_token_account
    create_token_account(
        &ctx.accounts.launchpad_state.to_account_info(),
        &ctx.accounts.signer.to_account_info(),
        &ctx.accounts.payment_token_account.to_account_info(),
        &ctx.accounts.payment_mint.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
        &ctx.accounts.token_program.to_account_info(),
        &[&[
            IDO_LAUNCHPAD_SEED.as_bytes(),
            ctx.accounts.launchpad_state.key().as_ref(),
            ctx.accounts.payment_mint.key().as_ref(),
            &[ctx.bumps.payment_token_account][..],
        ][..]],
    )?;

    ctx.accounts.launchpad_state.payment_mint = ctx.accounts.payment_mint.key();
    ctx.accounts.launchpad_state.meme_mint = ctx.accounts.meme_mint.key();
    ctx.accounts.launchpad_state.min_invest = min_invest;
    ctx.accounts.launchpad_state.max_invest = max_invest;
    ctx.accounts.launchpad_state.token_price = token_price;
    ctx.accounts.launchpad_state.start_time = start_time;
    ctx.accounts.launchpad_state.end_time = end_time;
    ctx.accounts.launchpad_state.admin = ctx.accounts.signer.key();
    ctx.accounts.launchpad_state.total_sold = 0;
    ctx.accounts.launchpad_state.claimed_amount = 0;
    ctx.accounts.launchpad_state.bump = ctx.bumps.launchpad_state;
    emit!(CreateLaunchpad {
        creator: ctx.accounts.signer.key(),
        meme_mint: ctx.accounts.meme_mint.key(),
        payment_mint: ctx.accounts.payment_mint.key(),
        min_invest,
        max_invest,
        token_price,
        start_time,
        end_time,
    });
    Ok(())
}
