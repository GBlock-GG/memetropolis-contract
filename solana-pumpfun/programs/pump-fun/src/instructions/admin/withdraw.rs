use crate::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::Token,
    token_interface::{Mint, TokenAccount},
};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    /// Admin address
    #[account(
      mut,
      address = crate::admin::id()
  )]
    pub authority: Signer<'info>,

    #[account(
    mut,
    mint::token_program = token_program,
  )]
    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

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
    associated_token::authority = authority,
    token::token_program = token_program,
  )]
    pub associted_user_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
impl Withdraw<'_> {
  pub fn apply(ctx: &mut Context<Withdraw>) -> Result<()> {

    let sol_amount = ctx.accounts.bonding_curve.to_account_info().lamports();
      //check sol_amount
      // assert!(
      //     sol_amount >= WITHDRAWABLE_MIN_SOL_AMOUNT,
      //     "Insufficient amount to withdraw"
      // );

    let token_amount = ctx.accounts.associted_bonding_curve.amount;
    //transfer token from vault to user
    let token_mint = ctx.accounts.token_mint.key();
    let vault_seeds = &[
        BONDING_CURVE_SEED.as_bytes(),
        token_mint.as_ref(),
        &[ctx.bumps.bonding_curve],
    ];
    let vault_signer_seeds = &[&vault_seeds[..]];
    let decimals = ctx.accounts.token_mint.decimals;
    transfer_token_from_vault_to_user(
        ctx.accounts.bonding_curve.to_account_info(),
        ctx.accounts.associted_bonding_curve.to_account_info(),
        ctx.accounts.associted_user_token_account.to_account_info(),
        ctx.accounts.token_mint.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        token_amount,
        decimals,
        vault_signer_seeds,
    )?;

    //transfer sol from vault to user
    transfer_sol_from_vault_to_user(
        ctx.accounts.bonding_curve.to_account_info(),
        ctx.accounts.authority.to_account_info(),
        sol_amount,
    )?;

    emit!(WithdrawEvent {
        mint: ctx.accounts.token_mint.key(),
        withdrawer: ctx.accounts.authority.key(),
        sol_output: sol_amount,
        token_output: token_amount,
    });

    Ok(())
  }
}

