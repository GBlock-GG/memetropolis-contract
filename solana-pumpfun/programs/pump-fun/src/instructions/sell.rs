use crate::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::Token,
    token_interface::{Mint, TokenAccount},
};


#[derive(Accounts)]
pub struct Sell<'info> {
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
    pub associted_bonding_curve: InterfaceAccount<'info, TokenAccount>,

  #[account(
    mut,
    associated_token::mint = token_mint,
    associated_token::authority = user,
    token::token_program = token_program,
  )]
  pub associted_user_token_account: InterfaceAccount<'info, TokenAccount>,

  #[account(
    mut,
    seeds = [
      USER_CONF_SEED,
      token_mint.key().as_ref(),
      user.key().as_ref(),
    ],
    bump,
  )]
  pub user_conf:Box<Account<'info, UserConf>>,

  #[account(mut)]
  pub user: Signer<'info>,
  pub associated_token_program: Program<'info, AssociatedToken>,
  pub token_program: Program<'info, Token>,
  pub system_program: Program<'info, System>,
}
impl Sell<'_> {
  pub fn apply(
    ctx: &mut Context<Sell>,
    amount: u64,         //sell token Amount
  ) -> Result<()> {
    require!(
      ctx.accounts.bonding_curve.launch_date <= Clock::get()?.unix_timestamp.try_into().unwrap(),
      PumpFunError::TokenNotLaunched,
    );

    let decimals = ctx.accounts.token_mint.decimals;

    // transfer token from user to vault
    transfer_token_from_user_to_vault(
        ctx.accounts.user.to_account_info(), //authority
        ctx.accounts.associted_user_token_account.to_account_info(), // sender user's token account
        ctx.accounts.associted_bonding_curve.to_account_info(),
        ctx.accounts.token_mint.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        amount,
        ctx.accounts.token_mint.decimals,
    )?;

    let token_total_supply = (1000 - ctx.accounts.bonding_curve.reserved_ratio) * ctx.accounts.bonding_curve.max_supply / 10000;
    let current_supply =
      token_total_supply - ctx.accounts.associted_bonding_curve.amount;

    let sol_amount = calculate_cost(current_supply - amount, amount, decimals, ctx.accounts.bonding_curve.k, ctx.accounts.bonding_curve.initial_price);

    //transfer sol from vault to user
    transfer_sol_from_vault_to_user(
        ctx.accounts.bonding_curve.to_account_info(),
        ctx.accounts.user.to_account_info(),
        sol_amount,
    )?;
    ctx.accounts.user_conf.bought_amount -= sol_amount;
    emit!(SellEvent {
        mint: ctx.accounts.token_mint.key(),
        token_input: amount,
        sol_output: sol_amount,
        seller: ctx.accounts.user.key()
    });
    Ok(())
  }
}

