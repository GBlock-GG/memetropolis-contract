use crate::*;

pub const USER_CONF_SEED: &[u8] = b"memetropolis_user_conf";

#[account]
#[derive(InitSpace)]
pub struct UserConf {
  pub bought_amount: u64
}