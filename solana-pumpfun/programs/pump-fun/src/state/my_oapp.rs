use crate::*;
use oapp::endpoint::{instructions::RegisterOAppParams, ID as ENDPOINT_ID};

#[account]
#[derive(InitSpace)]
pub struct OAppConfig {
    // immutable
    pub endpoint_program: Pubkey,
    pub bump: u8,
    // mutable
    pub admin: Pubkey,
}

impl OAppConfig {
  // todo: optimize
  pub fn init(
      &mut self,
      endpoint_program: Option<Pubkey>,
      admin: Pubkey,
      accounts: &[AccountInfo],
      oapp_signer: Pubkey,
  ) -> Result<()> {
      self.admin = admin;
      self.endpoint_program = if let Some(endpoint_program) = endpoint_program {
          endpoint_program
      } else {
          ENDPOINT_ID
      };

      // register oapp
      oapp::endpoint_cpi::register_oapp(
          self.endpoint_program,
          oapp_signer,
          accounts,
          &[OAPP_SEED, &[self.bump]],
          RegisterOAppParams { delegate: self.admin },
      )
  }

}

#[account]
#[derive(InitSpace)]
pub struct LzReceiveTypesAccounts {
    pub oapp_config: Pubkey,
}