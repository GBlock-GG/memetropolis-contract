use std::ops::DerefMut;
use crate::*;

#[derive(Accounts)]
pub struct CreateConfig<'info> {
  /// Admin address
  #[account(
    mut,
    address = crate::admin::id()
  )]
  pub authority: Signer<'info>,

  #[account(
    init,
    seeds=[
      CONFIG_SEED.as_bytes(),
      &authority.key.as_ref()
    ],
    bump,
    payer = authority,
    space = Config::LEN
  )]
  pub config: Account<'info, Config>,

  pub system_program: Program<'info, System>,
}
impl CreateConfig<'_> {
  pub fn apply(ctx: &mut Context<CreateConfig>, params: &CreateConfigParams) -> Result<()> {
    let config = ctx.accounts.config.deref_mut();
    config.initialized = true;
    config.authority = ctx.accounts.authority.key();
    config.fee_recipient = params.fee_recipient;
    Ok(())
  }
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct CreateConfigParams {
  pub fee_recipient: Pubkey,
}
