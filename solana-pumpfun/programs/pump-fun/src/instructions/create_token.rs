use crate::*;
// use crate::error::ErrorCode;
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
    signer,
    init,
    payer = payer,
    mint::decimals = 9,
    mint::authority = oft_config,
    mint::token_program = token_program,
  )]
  pub token_mint: InterfaceAccount<'info, Mint>,

  #[account(
    init,
    payer = payer,
    space = 8 + OftConfig::INIT_SPACE,
    seeds = [OFT_SEED, token_mint.key().as_ref()],
    bump
  )]
  pub oft_config: Account<'info, OftConfig>,

  #[account(
    init,
    payer = payer,
    space = 8 + LzReceiveTypesAccounts::INIT_SPACE,
    seeds = [LZ_RECEIVE_TYPES_SEED, &oft_config.key().as_ref()],
    bump
  )]
  pub lz_receive_types_accounts: Account<'info, LzReceiveTypesAccounts>,

  /// CHECK
  #[account(
    init,
    seeds = [
      BONDING_CURVE_SEED.as_bytes(),
      token_mint.key().as_ref()
    ],
    payer = payer,
    space = 8,
    bump,
  )]
  pub bonding_curve: UncheckedAccount<'info>,

  #[account(
    init,
    associated_token::mint = token_mint,
    associated_token::authority = bonding_curve,
    payer = payer,
    token::token_program = token_program,
  )]
  pub associted_bonding_curve: Box<InterfaceAccount<'info, TokenAccount>>,

  #[account(
    init,
    associated_token::mint = token_mint,
    associated_token::authority = payer,
    payer = payer,
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
    ctx.accounts.oft_config.bump = ctx.bumps.oft_config;
    ctx.accounts.oft_config.token_mint = ctx.accounts.token_mint.key();
    ctx.accounts.oft_config.token_program = ctx.accounts.token_program.key();
    ctx.accounts.lz_receive_types_accounts.oft_config = ctx.accounts.oft_config.key();
    ctx.accounts.lz_receive_types_accounts.token_mint = ctx.accounts.token_mint.key();
    let oapp_signer = ctx.accounts.oft_config.key();
    ctx.accounts.oft_config.init(
      params.endpoint_program,
      ctx.accounts.payer.key(),
      SHARED_DECIMALS,
      ctx.accounts.token_mint.decimals,
      ctx.remaining_accounts,
      oapp_signer,
    )?;

    let seeds = &[OFT_SEED, &ctx.accounts.token_mint.key().to_bytes(), &[ctx.bumps.oft_config]];
    let signer_seeds = [&seeds[..]];

    // create metadata account
    let cpi_context = CpiContext::new_with_signer(
      ctx.accounts.token_metadata_program.to_account_info(),
      CreateMetadataAccountsV3 {
          metadata: ctx.accounts.metadata.to_account_info(),
          mint: ctx.accounts.token_mint.to_account_info(),
          mint_authority: ctx.accounts.oft_config.to_account_info(),
          update_authority: ctx.accounts.oft_config.to_account_info(),
          payer: ctx.accounts.payer.to_account_info(),
          system_program: ctx.accounts.system_program.to_account_info(),
          rent: ctx.accounts.rent.to_account_info(),
      },
      &signer_seeds,
    );

    let data_v2 = DataV2 {
      name: params.name.clone(),
      symbol: params.symbol.clone(),
      uri: params.uri.clone(),
      seller_fee_basis_points: 0,
      creators: None,
      collection: None,
      uses: None,
    };

    create_metadata_accounts_v3(cpi_context, data_v2, false, true, None)?;

    //mint_to  MAX_SUPPLY to bonding curve
    let cpi_accounts = MintTo {
      mint: ctx.accounts.token_mint.to_account_info(),
      to: ctx.accounts.associted_bonding_curve.to_account_info(),
      authority: ctx.accounts.oft_config.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
    mint_to(cpi_context.with_signer(&signer_seeds), MAX_SUPPLY)?;

    emit!(CreateTokenEvent {
      creator: ctx.accounts.payer.key(),
      token_name: params.name.clone(),
      token_symbol: params.symbol.clone(),
      token_uri: params.uri.clone(),
      mint: ctx.accounts.token_mint.key()
    });

    Ok(())
  }
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct CreateTokenParams {
  pub name: String,
  pub symbol: String,
  pub uri: String,
  pub endpoint_program: Option<Pubkey>,
}



