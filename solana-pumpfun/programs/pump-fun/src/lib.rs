use anchor_lang::prelude::*;
mod instructions;
pub mod state;
pub mod utils;
mod events;
mod errors;

use instructions::*;
use state::*;
use utils::*;
use events::*;
use errors::*;


declare_id!("5LYPhunsULS1z59XE5nvaciuTYxa2M5foFMXhSKuXfv1");

pub mod admin {
    use anchor_lang::prelude::declare_id;
    #[cfg(feature = "devnet")]
    declare_id!("FukUMnHm7SUMMHFkgB5vXF4c8E1HP2ZxXKM9yDXHTYp");
    #[cfg(not(feature = "devnet"))]
    declare_id!("FukUMnHm7SUMMHFkgB5vXF4c8E1HP2ZxXKM9yDXHTYp");
}

#[program]
pub mod pump_fun {
  use super::*;

  pub fn create_config(mut ctx: Context<CreateConfig>, params: CreateConfigParams) -> Result<()> {
    assert!(params.default_decimals >= 6);
    assert!(params.max_supply >= 1000);
    assert!(params.init_supply >= 200);
    CreateConfig::apply(&mut ctx, &params)
  }

  pub fn update_config(mut ctx: Context<UpdateConfig>, params: CreateConfigParams) -> Result<()> {
    assert!(params.default_decimals >= 6);
    assert!(params.max_supply >= 1000);
    assert!(params.init_supply >= 200);
    UpdateConfig::apply(&mut ctx, &params)
  }

  // create meme token
  pub fn create_token(
      mut ctx: Context<CreateToken>,
      name: String,
      symbol: String,
      uri: String,
  ) -> Result<()> {
    CreateToken::apply(&mut ctx, name, symbol, uri)
  }

  pub fn buy(mut ctx: Context<Buy>, amount: u64, max_sol_cost: u64) -> Result<()> {
    Buy::apply(&mut ctx, amount, max_sol_cost)
  }

  pub fn sell(mut ctx: Context<Sell>, amount: u64, min_sol_output: u64) -> Result<()> {
    Sell::apply(&mut ctx, amount, min_sol_output)
  }

  pub fn withdraw(mut ctx: Context<Withdraw>) -> Result<()> {
    Withdraw::apply(&mut ctx)
  }
}
