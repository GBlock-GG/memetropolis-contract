use std::ops::DerefMut;
use crate::*;

#[derive(Accounts)]
pub struct CreateGlobalConfig<'info> {
  /// authority to withdraw from bonding_curve_account
  #[account(
    mut,
  )]
  pub payer: Signer<'info>,

  #[account(
    init,
    seeds=[
      CONFIG_SEED,
    ],
    bump,
    payer = payer,
    space = 8 + GlobalConfig::INIT_SPACE
  )]
  pub global_config: Account<'info, GlobalConfig>,

  pub system_program: Program<'info, System>,
}
impl CreateGlobalConfig<'_> {
  pub fn apply(ctx: &mut Context<CreateGlobalConfig>, params: &CreateGlobalConfigParams) -> Result<()> {
    let config = ctx.accounts.global_config.deref_mut();
    require!(!config.is_initialized, PumpFunError::AlreadyInitialized);
    config.admin = ctx.accounts.payer.key();
    config.fee_recipient = params.fee_recipient;
    config.fee_rate = params.fee_rate; // 1: 0.001 %
    config.bump = ctx.bumps.global_config;
    config.is_initialized = true;
    Ok(())
  }
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct CreateGlobalConfigParams {
  pub fee_recipient: Pubkey,
  pub admin: Pubkey,
  pub fee_rate: u32,
}
