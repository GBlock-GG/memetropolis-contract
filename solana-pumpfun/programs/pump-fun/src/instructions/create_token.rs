use anchor_lang::prelude::*;
// use crate::error::ErrorCode;
use anchor_spl::token::InitializeAccount3;

pub const TOKEN_SEED: &str = "pumpfun_token";
pub const TOKEN_MINT_AUTHORITY_SEED: &str = "pumpfun_mint_authority";


pub fn create_token(
  ctx: Context<CreateToken>,
  name: String,
  symbol: String,
  uri: String,
) -> Result<()> {
    
  Ok(())
}

#[derive(Accounts)]
pub struct CreateToken<'info> {
  #[account(
    init,
    seeds=[
      TOKEN_SEED.as_bytes(),
      &mint_authority.key.as_ref()
    ],
    payer = user,
    bump,
    space = 8 + 82
  )]
  pub token_mint: AccountInfo<'info>,
  #[account(
    seeds=[
      TOKEN_MINT_AUTHORITY_SEED.as_bytes(),
    ],
    bump
  )]
  pub mint_authority: AccountInfo<'info>,

  #[account(mut)]
  pub user: Signer<'info>,
  pub system_program: Program<'info, System>,  
}

