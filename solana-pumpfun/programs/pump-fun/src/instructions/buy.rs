use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::Token,
    token_interface::{Mint, TokenAccount},
};

use crate::states::*;
use crate::utils::*;

pub fn buy(
    ctx: Context<Buy>,
    amount: u64,       //buy Amount
    max_sol_cost: u64, // max Sol amount for slippage
) -> Result<()> {
    let decimals = ctx.accounts.token_mint.decimals;

    // check to ensure funding goal is not met
    assert!(
        ctx.accounts.associted_bonding_curve.amount > ctx.accounts.config.init_supply,
        "Funding Already Raised"
    );

    let available_qty =
        ctx.accounts.associted_bonding_curve.amount - ctx.accounts.config.init_supply;
    assert!(amount < available_qty, "Not enough available supply");

    let current_supply =
        ctx.accounts.config.max_supply - ctx.accounts.associted_bonding_curve.amount;
    let required_lamports = calculate_cost(current_supply, amount, decimals);

    assert!(
        max_sol_cost >= required_lamports,
        "Incorrect value of SOL sent"
    );

    //transfer sol to vault
    transfer_sol(
        ctx.accounts.user.to_account_info(),
        ctx.accounts.bonding_curve.to_account_info(),
        required_lamports,
    )?;
    //transfer fee

    //transfer token from vault to user
    let token_mint = ctx.accounts.token_mint.key();
    let vault_seeds = &[
        BONDING_CURVE_SEED.as_bytes(),
        token_mint.as_ref(),
        &[ctx.bumps.bonding_curve],
    ];
    let vault_signer_seeds = &[&vault_seeds[..]];

    transfer_token_from_vault_to_user(
        ctx.accounts.bonding_curve.to_account_info(),
        ctx.accounts.associted_bonding_curve.to_account_info(),
        ctx.accounts.associted_user_token_account.to_account_info(),
        ctx.accounts.token_mint.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        amount,
        decimals,
        vault_signer_seeds,
    )?;
    emit!(BuyEvent {
        mint: ctx.accounts.token_mint.key(),
        token_output: amount,
        sol_input: required_lamports,
        buyer: ctx.accounts.user.key()
    });
    Ok(())
}

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(
    mut,
    mint::authority = mint_authority,
    mint::token_program = token_program,
  )]
    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    /// CHECKED
    #[account(
    seeds = [
      TOKEN_MINT_AUTHORITY_SEED.as_bytes(),
      config.key().as_ref()
    ],
    bump
  )]
    pub mint_authority: UncheckedAccount<'info>,

    #[account(
    seeds = [
      CONFIG_SEED.as_bytes(),
      config.authority.as_ref(),
    ],
    bump
  )]
    pub config: Box<Account<'info, Config>>,

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
    pub associted_bonding_curve: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
    mut,
    associated_token::mint = token_mint,
    associated_token::authority = user,
    token::token_program = token_program,
  )]
    pub associted_user_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
