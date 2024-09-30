use std::ops::DerefMut;

use crate::states::*;
use anchor_lang::prelude::*;

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

pub fn create_config(
    ctx: Context<CreateConfig>,
    fee_recipient: Pubkey,
    max_supply: u64,
    init_supply: u64,
    default_decimals: u8,
) -> Result<()> {
    let config = ctx.accounts.config.deref_mut();
    config.initialized = true;
    config.authority = ctx.accounts.authority.key();
    config.fee_recipient = fee_recipient;
    config.max_supply = max_supply;
    config.init_supply = init_supply;
    config.default_decimals = default_decimals;
    Ok(())
}
