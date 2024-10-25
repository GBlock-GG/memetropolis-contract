
use anchor_lang::prelude::*;
mod instructions;
pub mod state;
pub mod utils;
mod events;
mod errors;
pub mod msg_codec;

use instructions::*;
use state::*;
use utils::*;
use events::*;
use errors::*;
// use oapp::{
//   endpoint::{MessagingFee, MessagingReceipt},
  // LzReceiveParams,
// };


declare_id!("BmgN6isDTS96BaY7fv82EQ7VHoeGZjvLMnLykfLrLeJN");

// pub const OFT_SEED: &[u8] = b"Oft";
pub const OAPP_SEED: &[u8] = b"OApp";
pub const PEER_SEED: &[u8] = b"Peer";
pub const ENFORCED_OPTIONS_SEED: &[u8] = b"EnforcedOptions";
pub const LZ_RECEIVE_TYPES_SEED: &[u8] = oapp::LZ_RECEIVE_TYPES_SEED;

// pub const SHARED_DECIMALS:u8 = 6;
pub const MAX_SUPPLY:u64 = 1000_000_000_000_000;
pub const INIT_SUPPLY:u64 = 200_000_000_000_000;

#[program]
pub mod pump_fun {
  use super::*;

  pub fn create_global_config(mut ctx: Context<CreateGlobalConfig>, params: CreateGlobalConfigParams) -> Result<()> {
    CreateGlobalConfig::apply(&mut ctx, &params)
  }

  pub fn update_global_config(mut ctx: Context<UpdateGlobalConfig>, params: UpdateGlobalConfigParams) -> Result<()> {
    UpdateGlobalConfig::apply(&mut ctx, &params)
  }

  pub fn init_oapp(mut ctx: Context<InitOApp>, params: InitOAppParams) -> Result<()> {
    InitOApp::apply(&mut ctx, &params)
  }

  // create meme token
  pub fn create_token(
      mut ctx: Context<CreateToken>,
      params: CreateTokenParams,
  ) -> Result<()> {
    CreateToken::apply(&mut ctx, &params)
  }

  pub fn buy(mut ctx: Context<Buy>, amount: u64, max_sol_cost: u64) -> Result<()> {
    Buy::apply(&mut ctx, amount, max_sol_cost)
  }

  pub fn buy_in_sol(mut ctx: Context<BuyInSol>, amount_min: u64, sol: u64) -> Result<()> {
    BuyInSol::apply(&mut ctx, amount_min, sol)
  }

  pub fn sell(mut ctx: Context<Sell>, amount: u64) -> Result<()> {
    Sell::apply(&mut ctx, amount)
  }

  pub fn withdraw(mut ctx: Context<Withdraw>) -> Result<()> {
    Withdraw::apply(&mut ctx)
  }
  // OFT
  // ============================== Admin ==============================
  pub fn transfer_oapp_admin(
    mut ctx: Context<TransferOAppAdmin>,
    params: TransferOAppAdminParams,
  ) -> Result<()> {
    TransferOAppAdmin::apply(&mut ctx, &params)
  }

  pub fn set_peer(mut ctx: Context<SetPeer>, params: SetPeerParams) -> Result<()> {
      SetPeer::apply(&mut ctx, &params)
  }

  pub fn set_enforced_options(
      mut ctx: Context<SetEnforcedOptions>,
      params: SetEnforcedOptionsParams,
  ) -> Result<()> {
      SetEnforcedOptions::apply(&mut ctx, &params)
  }

  // ============================== Public ==============================
  pub fn lz_receive(mut ctx: Context<LzReceive>, params: LzReceiveParams) -> Result<()> {
      LzReceive::apply(&mut ctx, &params)
  }

  pub fn lz_receive_types(
      ctx: Context<LzReceiveTypes>,
      params: LzReceiveParams,
  ) -> Result<Vec<oapp::endpoint_cpi::LzAccount>> {
      LzReceiveTypes::apply(&ctx, &params)
  }

}
