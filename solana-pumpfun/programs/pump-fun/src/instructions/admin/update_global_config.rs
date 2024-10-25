use std::ops::DerefMut;
use crate::*;

#[derive(Accounts)]
pub struct UpdateGlobalConfig<'info> {
    /// Admin address
  #[account(
    mut,
    address = global_config.admin
  )]
  pub authority: Signer<'info>,

  #[account(
    seeds=[
      CONFIG_SEED,
    ],
    bump = global_config.bump
  )]
  pub global_config: Account<'info, GlobalConfig>,
}

impl UpdateGlobalConfig<'_> {
  pub fn apply(ctx: &mut Context<UpdateGlobalConfig>, params: &UpdateGlobalConfigParams) -> Result<()> {
    let config = ctx.accounts.global_config.deref_mut();

    if params.fee_recipient.is_some() {
      config.fee_recipient = params.fee_recipient.unwrap();
    }
    if params.fee_rate.is_some() {
      config.fee_rate = params.fee_rate.unwrap();
    }
    Ok(())
  }
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct UpdateGlobalConfigParams {
  pub fee_recipient: Option<Pubkey>,
  pub fee_rate: Option<u32>,
}

