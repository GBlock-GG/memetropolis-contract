use anchor_lang::prelude::*;
// use crate::error::ErrorCode;
use anchor_spl::{
  associated_token::AssociatedToken,
  metadata::{
    create_metadata_accounts_v3,
    mpl_token_metadata::{
      accounts::Metadata as MetadataAccount,
      types::DataV2
    },
    CreateMetadataAccountsV3,
    Metadata
  },
  token::{self, mint_to, Token},
  token_interface::{Mint, TokenAccount},
};

use crate::states::*;

pub fn create_token(
  ctx: Context<CreateToken>,
  name: String,
  symbol: String,
  uri: String,
) -> Result<()> {
  let config_key = ctx.accounts.config.key();
  let seeds = &[
    TOKEN_MINT_AUTHORITY_SEED.as_bytes(),
    config_key.as_ref(),
    &[ctx.bumps.mint_authority]
  ];
  let signer_seeds = [&seeds[..]];
  
  // create metadata account
  let cpi_context = CpiContext::new_with_signer(
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
    &signer_seeds
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
  //mint_to
  let mint_to_cpi_context = CpiContext::new_with_signer(
    ctx.accounts.token_program.to_account_info(),
    token::MintTo {
      mint: ctx.accounts.token_mint.to_account_info(),
      to: ctx.accounts.associted_user_token_account.to_account_info(),
      authority: ctx.accounts.mint_authority.to_account_info(),
    },
    &signer_seeds
  );
  let decimals = (10 as u64).wrapping_pow(8);

  mint_to(
    mint_to_cpi_context,
    ctx.accounts.config.init_supply * decimals,
  )?;

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

  /// CHECK
  #[account(
    init_if_needed,
    seeds = [
      TOKEN_MINT_AUTHORITY_SEED.as_bytes(),
      config.key().as_ref()
    ],
    payer = user,
    space = 8,
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

  #[account(
    init,
    associated_token::mint = token_mint,
    associated_token::authority = user,
    payer = user,
    token::token_program = token_program,
  )]
  pub associted_user_token_account: InterfaceAccount<'info, TokenAccount>,
 
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

