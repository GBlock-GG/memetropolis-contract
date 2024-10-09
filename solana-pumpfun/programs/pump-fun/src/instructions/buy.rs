use crate::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token::Token,
    token_interface::{Mint, TokenAccount},
};

#[derive(Accounts)]
pub struct Buy<'info> {
  #[account(address = oft_config.token_mint)]
  pub token_mint: Box<InterfaceAccount<'info, Mint>>,

  #[account(
    seeds = [OFT_SEED, &get_oft_config_seed(&oft_config).to_bytes()],
    bump = oft_config.bump
  )]
  pub oft_config: Account<'info, OftConfig>,

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

impl Buy<'_> {
  pub fn apply(ctx: &mut Context<Buy>, amount: u64, max_sol_cost: u64) -> Result<()> {
    let decimals = ctx.accounts.token_mint.decimals;

    // check to ensure funding goal is not met
    require!(
        ctx.accounts.associted_bonding_curve.amount > INIT_SUPPLY,
        PumpFunError::AlreadyRaised
    );

    let available_qty =
        ctx.accounts.associted_bonding_curve.amount - INIT_SUPPLY;
    require!(amount < available_qty, PumpFunError::NotEnoughSuppply);

    let current_supply =
        MAX_SUPPLY - ctx.accounts.associted_bonding_curve.amount;
    let required_lamports = calculate_cost(current_supply, amount, decimals);

    require!(
        max_sol_cost >= required_lamports,
        PumpFunError::InvalidSolAmount
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
}
