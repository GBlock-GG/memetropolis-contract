use crate::utils::{create_token_account, transfer_from_launchpad, transfer_from_user};
use anchor_lang::{
    accounts::interface_account::InterfaceAccount, prelude::*, solana_program::clock,
};
use anchor_spl::{
    token::Token,
    token_interface::{Mint, TokenAccount},
};

pub mod utils;

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("GghUhzqc1sYRKC13vfDzTq5XazGveP4GqxTG52U12qLZ");

pub const IDO_LAUNCHPAD_SEED: &str = "ido_launchpad";
pub const IDO_LAUNCHPAD_STATE_LEN: usize = 8 + 32 * 3 + 8 * 2 + 16 * 5 + 1 * 1;
pub const USER_STAKE_LEN: usize = 8 + 16 + 8 + 1 * 3;

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
        Ok(())
    }
    //transfer user's token_account to launchpad token account
    pub fn buy_tokens(ctx: Context<BuyTokens>, amount: u64) -> Result<()> {
        let launchpad_state = &ctx.accounts.launchpad_state;
        let block_timestamp = clock::Clock::get()?.unix_timestamp as u64;
        if launchpad_state.start_time > block_timestamp
            || launchpad_state.end_time < block_timestamp
        {
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
        if (cost as u128) < launchpad_state.min_invest
            || (cost as u128) > launchpad_state.max_invest
        {
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
        ctx.accounts.user_stake.invests = ctx.accounts.user_stake.invests + cost;
        ctx.accounts.user_stake.purchased = ctx.accounts.user_stake.purchased + amount;
        ctx.accounts.launchpad_state.total_sold = ctx.accounts.launchpad_state.total_sold + amount;

        Ok(())
    }

    //transfer launchpad's meme token_account to user's token account
    pub fn claim_tokens(ctx: Context<ClaimTokens>) -> Result<()> {
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
        ctx.accounts.launchpad_state.claimed_amount =
            ctx.accounts.launchpad_state.claimed_amount + claim_amount;
        Ok(())
    }
    //called by admin
    //transfered from launchpad to beneficary for payment token
    pub fn withdraw_payment(ctx: Context<WithdrawFunds>) -> Result<()> {
        //check sale has ended
        let block_timestamp = clock::Clock::get()?.unix_timestamp as u64;
        if block_timestamp <= ctx.accounts.launchpad_state.end_time {
            return err!(ErrCode::InvalidSaleEnd);
        }

        let withdraw_amount: u64 = ctx.accounts.launchpad_payment_token_account.amount;
        transfer_from_launchpad(
            ctx.accounts.launchpad_state.to_account_info(),
            ctx.accounts
                .launchpad_payment_token_account
                .to_account_info(),
            ctx.accounts
                .beneficiary_payment_token_account
                .to_account_info(),
            ctx.accounts.payment_mint.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            withdraw_amount,
            ctx.accounts.payment_mint.decimals,
            &[&[
                IDO_LAUNCHPAD_SEED.as_bytes(),
                ctx.accounts.launchpad_state.meme_mint.as_ref(),
                ctx.accounts.launchpad_state.admin.key().as_ref(),
                &[ctx.accounts.launchpad_state.bump],
            ]],
        )?;
        Ok(())
    }

    pub fn withdraw_meme(ctx: Context<WithdrawMeme>) -> Result<()> {
        let block_timestamp = clock::Clock::get()?.unix_timestamp as u64;
        if block_timestamp < ctx.accounts.launchpad_state.end_time {
            return err!(ErrCode::InvalidSaleEnd);
        }
        let withdraw_amount = ctx.accounts.launchpad_meme_token_account.amount
            - (ctx.accounts.launchpad_state.total_sold
                - ctx.accounts.launchpad_state.claimed_amount);
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
        Ok(())
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
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
      init,
      seeds=[IDO_LAUNCHPAD_SEED.as_bytes(), meme_mint.key().as_ref(), signer.key.as_ref()],
      bump,
      payer = signer,
      space = IDO_LAUNCHPAD_STATE_LEN)
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
      space = USER_STAKE_LEN
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

#[derive(Accounts)]
pub struct ClaimTokens<'info> {
    #[account(
      mut,
      constraint = launchpad_state.payment_mint == payment_mint.key()
                    && launchpad_state.meme_mint == user_meme_token_account.mint
                    && launchpad_state.meme_mint == meme_mint.key()
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

#[derive(Accounts)]
pub struct WithdrawFunds<'info> {
    #[account(
      mut,
      constraint = launchpad_state.payment_mint == payment_mint.key()
                    && launchpad_state.admin == signer.key()
  )]
    pub launchpad_state: Account<'info, LaunchpadState>,

    #[account(
      mut,
      token::mint = payment_mint,
      seeds = [
          IDO_LAUNCHPAD_SEED.as_bytes(),
          launchpad_state.key().as_ref(),
          payment_mint.key().as_ref()
      ],
      bump
  )]
    pub launchpad_payment_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
      mut,
      token::mint = payment_mint
  )]
    pub beneficiary_payment_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
      mint::token_program = token_program,
  )]
    pub payment_mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawMeme<'info> {
    #[account(
      mut,
      constraint = launchpad_state.meme_mint == meme_mint.key()
                    && launchpad_state.admin == signer.key()
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

#[account]
pub struct UserStake {
    pub is_initialized: bool,
    pub has_claimed_tokens: bool,
    pub invests: u64,
    pub purchased: u64,
    pub bump: u8,
}

#[error_code]
pub enum ErrCode {
    #[msg("Invalid start or end time")]
    InvalidTime,
    #[msg("Invalid token price")]
    InvalidPrice,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Sale is not active")]
    InvalidSaleActive,
    #[msg("Sale has ended yet")]
    InvalidSaleEnd,
    #[msg("Tokens already claimed")]
    InvalidClaim,
    #[msg("RemainedNotClaim")]
    RemainedNotClaim,
}
