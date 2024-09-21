use anchor_lang::prelude::*;
// use crate::error::ErrorCode;
use anchor_spl::{
  associated_token::AssociatedToken,
  token_interface::{ Mint, TokenAccount },
  token::Token,
  metadata::{
    Metadata,
    create_metadata_accounts_v3,
    CreateMetadataAccountsV3,
    mpl_token_metadata::{
      types::DataV2,
      accounts::Metadata as MetadataAccount,
    },
  }
};


use crate::states::*;

pub const TOKEN_SEED: &str = "pumpfun_token";
pub const TOKEN_MINT_AUTHORITY_SEED: &str = "pumpfun_mint_authority";
pub const BONDING_CURVE_SEED: &str = "pumpfun_bonding_curve";

pub fn create_token(
  ctx: Context<CreateToken>,
  name: String,
  symbol: String,
  uri: String,
) -> Result<()> {
  // create metadata account
  let cpi_context = CpiContext::new(
    ctx.accounts.token_metadata_program.to_account_info(),
    CreateMetadataAccountsV3 {
        metadata: ctx.accounts.metadata.to_account_info(),
        mint: ctx.accounts.token_mint.to_account_info(),
        mint_authority: ctx.accounts.mint_authority.to_account_info(),
        update_authority: ctx.accounts.mint_authority.to_account_info(),
        payer: ctx.accounts.user.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    },
  );

  let data_v2 = DataV2 {
    name: name,
    symbol: symbol,
    uri: uri,
    seller_fee_basis_points: 0,
    creators: None,
    collection: None,
    uses: None,
  };
  
  create_metadata_accounts_v3(cpi_context, data_v2, false, true, None)?;

  Ok(())
}

#[derive(Accounts)]
pub struct CreateToken<'info> {
  #[account(
    signer,
    init,
    payer = user,
    mint::decimals = config.default_decimals,
    mint::authority = mint_authority,
    mint::token_program = token_program,
  )]
  pub token_mint: InterfaceAccount<'info, Mint>,

  #[account(
    seeds = [
      TOKEN_MINT_AUTHORITY_SEED.as_bytes(),
      config.key().as_ref()
    ],
    seeds::program = system_program,
    bump
  )]
  pub mint_authority: SystemAccount<'info>,

  #[account(
    seeds = [
      CONFIG_SEED.as_bytes(),
      config.authority.as_ref(),
    ],
    bump
  )]
  pub config: Account<'info, Config>,

  /// CHECK
  #[account(
    init,
    seeds = [
      BONDING_CURVE_SEED.as_bytes(),
      token_mint.key().as_ref()
    ],
    payer = user,
    space = 8,
    bump,
  )]
  pub bonding_curve: UncheckedAccount<'info>,

  #[account(
    init,
    associated_token::mint = token_mint,
    associated_token::authority = bonding_curve,
    payer = user,
    token::token_program = token_program,
  )]
  pub associted_bonding_curve: InterfaceAccount<'info, TokenAccount>,
  
  /// CHECK
  #[account(
    mut,
    address = MetadataAccount::find_pda(&token_mint.key()).0   
  )]
  pub metadata: UncheckedAccount<'info>,

  #[account(mut)]
  pub user: Signer<'info>,
  pub associated_token_program: Program<'info, AssociatedToken>,
  pub token_program: Program<'info, Token>,
  pub token_metadata_program: Program<'info, Metadata>,
  pub rent: Sysvar<'info, Rent>,
  pub system_program: Program<'info, System>,  
}

