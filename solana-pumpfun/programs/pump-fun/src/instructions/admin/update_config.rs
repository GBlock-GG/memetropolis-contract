use std::ops::DerefMut;
use crate::*;

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    /// Admin address
  #[account(
    mut,
    address = crate::admin::id()
  )]
  pub authority: Signer<'info>,

  #[account(
    seeds=[
      CONFIG_SEED.as_bytes(),
      &authority.key.as_ref()
    ],
    bump,
  )]
  pub config: Account<'info, Config>,
}

impl UpdateConfig<'_> {
  pub fn apply(ctx: &mut Context<UpdateConfig>, params: &CreateConfigParams) -> Result<()> {
    let config = ctx.accounts.config.deref_mut();

    config.initialized = true;
    config.authority = ctx.accounts.authority.key();
    config.fee_recipient = params.fee_recipient;
    Ok(())
  }
}

