pub mod states;
pub mod instructions;
pub mod utils;

use anchor_lang::prelude::*;


use instructions::*;

declare_id!("DT2ovnYP2YHS6DWEQJ9ARHgznY97cydTcmkGAESfaagC");

#[program]
pub mod pump_fun {
  use super::*;
  
  pub fn create_config(
    ctx: Context<CreateConfig>,
    fee_recipient: Pubkey,
    max_supply: u64,
    init_supply: u64,
    default_decimals: u8
  ) -> Result<()> {
    assert!(default_decimals >= 6);
    assert!(max_supply >= 1000);
    assert!(init_supply >= 200);

    instructions::create_config(
      ctx,
      fee_recipient,
      max_supply,
      init_supply,
      default_decimals
    
    )
  }

  pub fn update_config(
    ctx: Context<UpdateConfig>,
    fee_recipient: Pubkey,
    max_supply: u64,
    init_supply: u64,
    default_decimals: u8
  ) -> Result<()> {
    assert!(default_decimals >= 6);
    assert!(max_supply >= 1000);
    assert!(init_supply >= 200);

    instructions::update_config(
      ctx,
      fee_recipient,
      max_supply,
      init_supply,
      default_decimals    
    )
  }

  // create meme token
  pub fn create_token(
    ctx: Context<CreateToken>,
    name: String,
    symbol: String,
    uri: String
  ) -> Result<()> {
    instructions::create_token(ctx, name, symbol, uri)
  }

  pub fn buy(ctx:Context<Buy>, amount: u64, max_sol_cost: u64) -> Result<()> {
    instructions::buy(ctx, amount, max_sol_cost)
  }

  pub fn sell(ctx:Context<Sell>, amount: u64, min_sol_output: u64) -> Result<()> {
    instructions::sell(ctx,amount, min_sol_output)
  }
  
}
