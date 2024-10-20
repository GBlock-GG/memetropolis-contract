use std::ops::DerefMut;
use crate::*;

#[derive(Accounts)]
pub struct CreateConfig<'info> {
  /// Admin address
  #[account(
    mut,
  )]
  pub payer: Signer<'info>,

  #[account(
    init,
    seeds=[
      CONFIG_SEED.as_bytes(),
    ],
    bump,
    payer = payer,
    space = 8 + GlobalConfig::INIT_SPACE
  )]
  pub global_config: Account<'info, GlobalConfig>,

  pub system_program: Program<'info, System>,
}
impl CreateConfig<'_> {
  pub fn apply(ctx: &mut Context<CreateConfig>, params: &CreateConfigParams) -> Result<()> {
    let config = ctx.accounts.global_config.deref_mut();
    config.admin = params.admin;
    config.fee_recipient = params.fee_recipient;
    config.fee_rate = params.fee_rate; // 1: 0.001 %
    config.bump = ctx.bumps.global_config;
    Ok(())
  }
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct CreateConfigParams {
  pub fee_recipient: Pubkey,
  pub admin: Pubkey,
  pub fee_rate: u32,
}
