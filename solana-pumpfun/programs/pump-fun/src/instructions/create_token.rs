use std::ops::DerefMut;

use crate::*;
use anchor_spl::{
  associated_token::AssociatedToken,
  metadata::{
      create_metadata_accounts_v3,
      mpl_token_metadata::{accounts::Metadata as MetadataAccount, types::DataV2},
      CreateMetadataAccountsV3, Metadata,
  },
  token::Token,
  token_interface::{Mint, MintTo, TokenAccount, mint_to},
};

#[derive(Accounts)]
pub struct CreateToken<'info> {
  #[account(mut)]
  pub payer: Signer<'info>,

  #[account(
    init,
    payer = payer,
    mint::decimals = 9,
    mint::authority = bonding_curve,
    mint::token_program = token_program,
  )]
  pub token_mint: Box<InterfaceAccount<'info, Mint>>,

  /// CHECK
  #[account(
    init,
    seeds = [
      BONDING_CURVE_SEED,
      token_mint.key().as_ref()
    ],
    payer = payer,
    space = 0,
    bump,
  )]
  pub bonding_curve: Box<Account<'info, BondingCurve>>,

  #[account(
    init,
    // mut,
    associated_token::mint = token_mint,
    associated_token::authority = bonding_curve,
    payer = payer,
    token::token_program = token_program,
  )]
  pub associted_bonding_curve: Box<InterfaceAccount<'info, TokenAccount>>,

  #[account(
    // init,
    associated_token::mint = token_mint,
    associated_token::authority = payer,
    // payer = payer,
    token::token_program = token_program,
  )]
  pub associted_user_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

  /// CHECK
  #[account(
    mut,
    address = MetadataAccount::find_pda(&token_mint.key()).0
  )]
  pub metadata: UncheckedAccount<'info>,

  pub associated_token_program: Program<'info, AssociatedToken>,
  pub token_program: Program<'info, Token>,
  pub token_metadata_program: Program<'info, Metadata>,
  pub rent: Sysvar<'info, Rent>,
  pub system_program: Program<'info, System>,
}

impl CreateToken<'_> {
  pub fn apply(
    ctx: &mut Context<CreateToken>,
    params: &CreateTokenParams,
  ) -> Result<()> {

    let seeds = &[BONDING_CURVE_SEED, &ctx.accounts.token_mint.key().to_bytes(), &[ctx.bumps.bonding_curve]];
    let signer_seeds = [&seeds[..]];

    // create metadata account
    let cpi_context = CpiContext::new_with_signer(
      ctx.accounts.token_metadata_program.to_account_info(),
      CreateMetadataAccountsV3 {
          metadata: ctx.accounts.metadata.to_account_info(),
          mint: ctx.accounts.token_mint.to_account_info(),
          mint_authority: ctx.accounts.bonding_curve.to_account_info(),
          update_authority: ctx.accounts.bonding_curve.to_account_info(),
          payer: ctx.accounts.payer.to_account_info(),
          system_program: ctx.accounts.system_program.to_account_info(),
          rent: ctx.accounts.rent.to_account_info(),
      },
      &signer_seeds,
    );

    let data_v2 = DataV2 {
      name: String::from_utf8(params.name.clone()).unwrap(),
      symbol: String::from_utf8(params.symbol.clone()).unwrap(),
      uri: String::from_utf8(params.uri.clone()).unwrap(),
      seller_fee_basis_points: 0,
      creators: None,
      collection: None,
      uses: None,
    };

    create_metadata_accounts_v3(cpi_context, data_v2, false, true, None)?;

    let params_max_supply = if params.max_supply == 0 { DEFAULT_MAX_SUPPLY } else { params.max_supply };


    let params_reserved_ratio = if params.reserved_ratio == 0 { DEFAULT_RESERVED_RATIO } else { params.reserved_ratio };
    let mut reserved_supply = 0;
    if params_reserved_ratio > 0 {
      reserved_supply = params_reserved_ratio * params_max_supply / 10000;
    }


    // mint_to  MAX_SUPPLY-RESERVED_SUPPLY to bonding curve
    let cpi_accounts = MintTo {
      mint: ctx.accounts.token_mint.to_account_info(),
      to: ctx.accounts.associted_bonding_curve.to_account_info(),
      authority: ctx.accounts.bonding_curve.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
    mint_to(cpi_context.with_signer(&signer_seeds), params_max_supply - reserved_supply)?;

    // mint_to RESERVED_SUPPLY to payer
    if reserved_supply > 0 {
      let cpi_accounts = MintTo {
        mint: ctx.accounts.token_mint.to_account_info(),
        to: ctx.accounts.associted_user_token_account.to_account_info(),
        authority: ctx.accounts.bonding_curve.to_account_info(),
      };
      let cpi_program = ctx.accounts.token_program.to_account_info();
      let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
      mint_to(cpi_context.with_signer(&signer_seeds),  reserved_supply)?;
    }

    let params_reserved_ratio = if params.reserved_ratio == 0 { DEFAULT_RESERVED_RATIO } else { params.reserved_ratio };

    let bonding_curve = ctx.accounts.bonding_curve.deref_mut();
    bonding_curve.bump = ctx.bumps.bonding_curve;
    let params_k = if params.k == 0.0 { DEFAULT_K } else {params.k};
    bonding_curve.k = params_k;

    let params_initial_price = if params.initial_price == 0 { DEFAULT_INITIAL_PRICE } else { params.initial_price };
    bonding_curve.initial_price = params_initial_price;

    bonding_curve.max_supply = params_max_supply;

    let params_sales_ratio = if params.sales_ratio == 0 { DEFAULT_SALES_RATIO } else { params.sales_ratio };
    bonding_curve.sales_ratio = params_sales_ratio;
    bonding_curve.reserved_ratio = params_reserved_ratio;
    let params_liquidity_pool_ratio = if params.liquidity_pool_ratio == 0 { DEFAULT_LIQUIDITY_RATIO } else { params.liquidity_pool_ratio };
    bonding_curve.liquidity_pool_ratio = params_liquidity_pool_ratio;
    bonding_curve.launch_date = params.launch_date; //Clock::get()?.unix_timestamp.try_into().unwrap();
    bonding_curve.maximum_per_user = params.maximum_per_user;

    emit!(CreateTokenEvent {
      creator: ctx.accounts.payer.key(),
      token_name: String::from_utf8(params.name.to_vec()).unwrap(),
      token_symbol: String::from_utf8(params.symbol.to_vec()).unwrap(),
      token_uri: String::from_utf8(params.uri.to_vec()).unwrap(),
      mint: ctx.accounts.token_mint.key(),
      k: bonding_curve.k,
      initial_price: bonding_curve.initial_price,
      max_supply: bonding_curve.max_supply,
      sales_ratio: bonding_curve.sales_ratio,
      reserved_ratio: bonding_curve.reserved_ratio,

      launch_date: bonding_curve.launch_date,
      maximum_per_user: bonding_curve.maximum_per_user
    });

    Ok(())
  }
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct CreateTokenParams {
  pub name: Vec<u8>,
  pub symbol: Vec<u8>,
  pub uri: Vec<u8>,
  pub k: f64,
  pub initial_price: u64,
  pub max_supply: u64,
  pub sales_ratio: u64,
  pub reserved_ratio: u64,
  pub liquidity_pool_ratio: u64, // 1%: 100
  pub launch_date: u64,  //timestamp that can buy
  pub maximum_per_user: u64, //lamports that user can buy max.
  // pub endpoint_program: Option<Pubkey>,
}



