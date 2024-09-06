pub mod utils;

use anchor_lang::{
    accounts::interface_account::InterfaceAccount, prelude::*, solana_program::clock,
};
use anchor_spl::{
    token::Token,
    token_interface::{Mint, TokenAccount},
};
use crate::utils::token::{create_token_account, transfer_from_launchpad, transfer_from_user};
// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("2BgPu9XwVRDtXe1gzRzAo1npLntV55EsnwjJL6itqZGb");

pub const IDO_LAUNCHPAD_SEED: &str = "ido_launchpad";
pub const IDO_AUTH_SEED: &str = "ido_launchpad_auth";
pub const IDO_LAUNCHPAD_STATE_LEN: usize = 8 + 3 * 32 + 2 * 8 + 3 * 16;

#[program]
mod ido_launchpad {
    use super::*;
    pub fn initialize(
        ctx: Context<Initialize>,
        min_invest: u128,
        max_invest: u128,
        token_price: u128,
        start_time: u64,
        end_time: u64,
    ) -> Result<()> {
        let block_timestamp = clock::Clock::get()?.unix_timestamp as u64;
        if start_time >= end_time || start_time <= block_timestamp || end_time <= block_timestamp {
            return err!(ErrorCode::InvalidTime);
        }
        if token_price == 0 {
            return err!(ErrorCode::InvalidPrice);
        }
        //create meme_token_account
        create_token_account(
            &ctx.accounts.authority.to_account_info(),
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
            &ctx.accounts.authority.to_account_info(),
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
        ctx.accounts.launchpad_state.authority_bump = ctx.bumps.authority;
        ctx.accounts.launchpad_state.owner = ctx.accounts.signer.key();

        Ok(())
    }
    //transfer user's token_account to launchpad token account
    pub fn buy_tokens(ctx: Context<BuyTokens>, amount: u64) -> Result<()> {
        if amount == 0 {
            return err!(ErrorCode::InvalidAmount);
        }
        transfer_from_user(
            ctx.accounts.signer.to_account_info(),
            ctx.accounts.user_payment_token_account.to_account_info(),
            ctx.accounts
                .launchpad_payment_token_account
                .to_account_info(),
            ctx.accounts.payment_mint.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            amount,
            ctx.accounts.payment_mint.decimals,
        )?;
        Ok(())
    }

    //transfer launchpad's meme token_account to user's token account
    pub fn claim_tokens(ctx: Context<ClaimTokens>) -> Result<()> {
        let claim_amount: u64 = 0;
        transfer_from_launchpad(
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.launchpad_meme_token_account.to_account_info(),
            ctx.accounts.user_meme_token_account.to_account_info(),
            ctx.accounts.meme_mint.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            claim_amount,
            ctx.accounts.meme_mint.decimals,
            &[&[
                IDO_AUTH_SEED.as_bytes(),
                &[ctx.accounts.launchpad_state.authority_bump],
            ]],
        )?;
        Ok(())
    }
    //called by admin
    //transfered from launchpad to beneficary for payment token
    pub fn withdraw_payment(ctx: Context<WithdrawFunds>) -> Result<()> {
        let withdraw_amount: u64 = 0;
        transfer_from_launchpad(
            ctx.accounts.authority.to_account_info(),
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
                IDO_AUTH_SEED.as_bytes(),
                &[ctx.accounts.launchpad_state.authority_bump],
            ]],
        )?;
        Ok(())
    }

    pub fn withdraw_meme(ctx: Context<WithdrawMeme>) -> Result<()> {
        let withdraw_amount: u64 = 0;
        transfer_from_launchpad(
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.launchpad_meme_token_account.to_account_info(),
            ctx.accounts
                .beneficiary_meme_token_account
                .to_account_info(),
            ctx.accounts.meme_mint.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            withdraw_amount,
            ctx.accounts.meme_mint.decimals,
            &[&[
                IDO_AUTH_SEED.as_bytes(),
                &[ctx.accounts.launchpad_state.authority_bump],
            ]],
        )?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    // We must specify the space in order to initialize an account.
    // First 8 bytes are default account discriminator,
    // next 8 bytes come from NewAccount.data being type u64.
    // (u64 = 64 bits unsigned integer = 8 bytes)
    #[account(
        init,
        seeds=[IDO_LAUNCHPAD_SEED.as_bytes(), signer.key.as_ref()],
        bump,
        payer = signer,
        space = IDO_LAUNCHPAD_STATE_LEN)
    ]
    pub launchpad_state: Account<'info, LaunchpadState>,

    #[account(
        seeds = [
            IDO_AUTH_SEED.as_bytes()
        ],
        bump,
    )]
    pub authority: UncheckedAccount<'info>,

    //create token account for meme_mint
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

    //create token account for payment_mint
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
        seeds = [ IDO_LAUNCHPAD_SEED.as_bytes(), admin.key.as_ref()],
        bump
    )]
    pub launchpad_state: Account<'info, LaunchpadState>,

    pub admin: UncheckedAccount<'info>,
    #[account(
        mut,
        token::authority = signer,
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
        constraint = payment_mint.key() == launchpad_state.payment_mint
    )]
    pub payment_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mint::token_program = token_program,
        constraint = meme_mint.key() == launchpad_state.meme_mint
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
        seeds = [ IDO_LAUNCHPAD_SEED.as_bytes(), admin.key.as_ref()],
        bump
    )]
    pub launchpad_state: Account<'info, LaunchpadState>,

    pub admin: UncheckedAccount<'info>,

    #[account(
        mut,
        token::authority = signer,
        token::mint = meme_mint
    )]
    pub user_meme_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        token::authority = authority,
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
        constraint = payment_mint.key() == launchpad_state.payment_mint
    )]
    pub payment_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mint::token_program = token_program,
        constraint = meme_mint.key() == launchpad_state.meme_mint
    )]
    pub meme_mint: InterfaceAccount<'info, Mint>,

    #[account(
        seeds = [
            IDO_AUTH_SEED.as_bytes()
        ],
        bump,
    )]
    pub authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawFunds<'info> {
    #[account(
        seeds = [
            IDO_LAUNCHPAD_SEED.as_bytes()
        ],
        bump,
    )]
    pub authority: UncheckedAccount<'info>,

    #[account(
        seeds = [ IDO_LAUNCHPAD_SEED.as_bytes(), admin.key.as_ref()],
        bump
    )]
    pub launchpad_state: Account<'info, LaunchpadState>,

    pub admin: UncheckedAccount<'info>,

    #[account(
        mut,
        token::authority = authority,
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
        constraint = payment_mint.key() == launchpad_state.payment_mint
    )]
    pub payment_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        constraint = signer.key() == launchpad_state.owner
    )]
    pub signer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawMeme<'info> {
    #[account(
        seeds = [
            IDO_LAUNCHPAD_SEED.as_bytes()
        ],
        bump,
    )]
    pub authority: UncheckedAccount<'info>,

    #[account(
        seeds = [ IDO_LAUNCHPAD_SEED.as_bytes(), admin.key.as_ref()],
        bump
    )]
    pub launchpad_state: Account<'info, LaunchpadState>,

    pub admin: UncheckedAccount<'info>,

    #[account(
        mut,
        token::authority = authority,
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
        constraint = meme_mint.key() == launchpad_state.meme_mint
    )]
    pub meme_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        constraint = signer.key() == launchpad_state.owner
    )]
    pub signer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct LaunchpadState {
    pub owner: Pubkey,
    pub meme_mint: Pubkey,
    pub payment_mint: Pubkey,
    pub start_time: u64,
    pub end_time: u64,
    pub max_invest: u128,
    pub min_invest: u128,
    pub token_price: u128,
    pub authority_bump: u8,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid start or end time")]
    InvalidTime,
    #[msg("Invalid token price")]
    InvalidPrice,
    #[msg("Invalid amount")]
    InvalidAmount,
}
