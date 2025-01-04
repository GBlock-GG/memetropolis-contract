use crate::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token::Token,
    token_interface::{Mint, TokenAccount},
};

#[derive(Accounts)]
pub struct Buy<'info> {

  pub token_mint: Box<InterfaceAccount<'info, Mint>>,

  #[account(
    seeds = [
      CONFIG_SEED,
    ],
    bump = global_config.bump
  )]
  pub global_config: Box<Account<'info, GlobalConfig>>,

    /// CHECK
  #[account(
    mut,
    seeds = [
      BONDING_CURVE_SEED,
      token_mint.key().as_ref()
    ],
    bump,
  )]
  pub bonding_curve: Box<Account<'info, BondingCurve>>,

  #[account(
    mut,
    associated_token::mint = token_mint,
    associated_token::authority = bonding_curve,
    token::token_program = token_program,
  )]
  pub associted_bonding_curve: Box<InterfaceAccount<'info, TokenAccount>>,

  #[account(
    init_if_needed,
    associated_token::mint = token_mint,
    associated_token::authority = user,
    token::token_program = token_program,
    payer = user,
  )]
  pub associted_user_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

  #[account(
    init_if_needed,
    seeds = [
      USER_CONF_SEED,
      token_mint.key().as_ref(),
      user.key().as_ref(),
    ],
    payer = user,
    bump,
    space = 8 + UserConf::INIT_SPACE
  )]
  pub user_conf:Box<Account<'info, UserConf>>,

  #[account(mut)]
    pub user: Signer<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl Buy<'_> {
  pub fn apply(ctx: &mut Context<Buy>, amount: u64, max_sol_cost: u64) -> Result<()> {
    require!(
      ctx.accounts.bonding_curve.launch_date <= Clock::get()?.unix_timestamp.try_into().unwrap(),
      PumpFunError::TokenNotLaunched,
    );

    let decimals = ctx.accounts.token_mint.decimals;

    // check to ensure funding goal is not met
    let init_supply = ctx.accounts.bonding_curve.liquidity_pool_ratio * ctx.accounts.bonding_curve.max_supply / 10000;
    require!(
        ctx.accounts.associted_bonding_curve.amount > init_supply,
        PumpFunError::AlreadyRaised
    );

    let available_qty =
        ctx.accounts.associted_bonding_curve.amount - init_supply;
    require!(amount <= available_qty, PumpFunError::NotEnoughSuppply);

    let token_total_supply = (1000 - ctx.accounts.bonding_curve.reserved_ratio) * ctx.accounts.bonding_curve.max_supply / 10000;
    let current_supply =
      token_total_supply - ctx.accounts.associted_bonding_curve.amount;
    let required_lamports = calculate_cost(
      current_supply,
      amount,
      decimals,ctx.accounts.bonding_curve.k,
      ctx.accounts.bonding_curve.initial_price
    );
    if ctx.accounts.bonding_curve.maximum_per_user > 0 {
      require!(
        ctx.accounts.user_conf.bought_amount + required_lamports <= ctx.accounts.bonding_curve.maximum_per_user,
        PumpFunError::UserBuyLimitExceed
      )
    }

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
        BONDING_CURVE_SEED,
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
    ctx.accounts.user_conf.bought_amount += required_lamports;

    emit!(BuyEvent {
        mint: ctx.accounts.token_mint.key(),
        token_output: amount,
        sol_input: required_lamports,
        buyer: ctx.accounts.user.key()
    });
    Ok(())
  }
}
