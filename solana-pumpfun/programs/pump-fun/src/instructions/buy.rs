use anchor_lang::prelude::*;
use crate::error::ErrorCode;

pub fn buy(ctx: Context<Buy>) -> Result<()> {
  Ok(())
}

#[derive(Accounts)]
pub struct Buy<'info> {
  #[account(mut)]
  pub user: Signer<'info>
}