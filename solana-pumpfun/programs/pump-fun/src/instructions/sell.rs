use anchor_lang::prelude::*;
use anchor_spl::{
  associated_token::AssociatedToken,
  token_interface::{ Mint, TokenAccount },
  token::Token,
};
use crate::error::ErrorCode;
use crate::states::*;
use crate::utils::*;

pub fn sell(
  ctx: Context<Sell>,
  amount: u64,  //sell token Amount
  min_sol_output: u64 // max Sol amount for slippage
) -> Result<()> {
  let decimals = (10 as u64).pow(ctx.accounts.token_mint.decimals as u32);

  // transfer token from user to vault
  transfer_token_from_user_to_vault(
    ctx.accounts.user.to_account_info(),//authority
    ctx.accounts.associted_user_token_account.to_account_info(), // sender user's token account
    ctx.accounts.associted_bonding_curve.to_account_info(),
    ctx.accounts.token_mint.to_account_info(),
    ctx.accounts.token_program.to_account_info(),
    amount,
    ctx.accounts.token_mint.decimals
  )?;
  let sol_amount = amount * INITIAL_PRICE / decimals;
  assert!(sol_amount >= min_sol_output, "Incorrect value of SOL sent");

  //transfer sol from vault to user
  transfer_sol_from_vault_to_user(
    ctx.accounts.bonding_curve.to_account_info(),
    ctx.accounts.user.to_account_info(),
    sol_amount,
  )?;
  Ok(())
}

#[derive(Accounts)]
pub struct Sell<'info> {

  #[account(
    mint::token_program = token_program,
  )]
  pub token_mint: InterfaceAccount<'info, Mint>,
  
  /// CHECK
  #[account(
    mut,
    seeds = [
      BONDING_CURVE_SEED.as_bytes(),
      token_mint.key().as_ref()
    ],
    bump,
  )]
  pub bonding_curve: UncheckedAccount<'info>,
  
  #[account(
    mut,
    associated_token::mint = token_mint,
    associated_token::authority = bonding_curve,
    token::token_program = token_program,
  )]
  pub associted_bonding_curve: InterfaceAccount<'info, TokenAccount>,
  
  #[account(
    mut,
    associated_token::mint = token_mint,
    associated_token::authority = user,
    token::token_program = token_program,
  )]
  pub associted_user_token_account: InterfaceAccount<'info, TokenAccount>,

  #[account(mut)]
  pub user: Signer<'info>,
  pub associated_token_program: Program<'info, AssociatedToken>,
  pub token_program: Program<'info, Token>,
  pub system_program: Program<'info, System>
}