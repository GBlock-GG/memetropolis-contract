use crate::*;

#[event]
pub struct CreateTokenEvent {
  pub creator: Pubkey,
  pub token_name: String,
  pub token_symbol: String,
  pub token_uri: String,
  pub mint: Pubkey,
  pub k: f64,
  pub initial_price: u64,
  pub max_supply: u64,
  pub sales_ratio: u64,
  pub reserved_ratio: u64,
  pub launch_date: u64,
  pub maximum_per_user: u64,
}

#[event]
pub struct BuyEvent {
  pub mint: Pubkey,
  pub buyer: Pubkey,
  pub sol_input: u64,
  pub token_output: u64,
}

#[event]
pub struct SellEvent {
  pub mint: Pubkey,
  pub seller: Pubkey,
  pub sol_output: u64,
  pub token_input: u64,
}

#[event]
pub struct WithdrawEvent {
  pub mint: Pubkey,
  pub withdrawer: Pubkey,
  pub sol_output: u64,
  pub token_output: u64,
}

#[event]
pub struct OFTSent {
    pub guid: [u8; 32],
    pub dst_eid: u32,
    pub from: Pubkey,
    pub amount_sent_ld: u64,
    pub amount_received_ld: u64,
}

#[event]
pub struct OFTReceived {
    pub guid: [u8; 32],
    pub src_eid: u32,
    pub to: Pubkey,
    pub amount_received_ld: u64,
}
