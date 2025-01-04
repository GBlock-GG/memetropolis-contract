use crate::*;

#[account]
#[derive(InitSpace)]
pub struct BondingCurve {
  pub k:f64,
  pub initial_price: u64,
  pub max_supply: u64,
  pub sales_ratio: u64,
  pub reserved_ratio: u64, // percent 1%: 100
  pub liquidity_pool_ratio: u64, //percent 1%:100
  pub launch_date: u64,
  pub maximum_per_user:u64, // lamports for total max buy amount
  pub bump:u8,
}