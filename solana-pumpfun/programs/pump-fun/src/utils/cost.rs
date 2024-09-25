use crate::utils::convert_to_float;

pub const K: f64 = 0.000005;
pub const INITIAL_PRICE:u64 = 8000; // 0.000008  SOL / per token
pub fn calculate_cost(current_supply: u64, tokens_to_buy: u64, decimals: u8) -> u64 {
    // // Calculate the exponent parts scaled to avoid precision loss
    let exponent1 = K * convert_to_float(current_supply + tokens_to_buy, decimals);
    let exponent2 = K * convert_to_float(current_supply, decimals);

    // // Calculate e^(kx) using the exp function
    let exp1 = exp(exponent1);
    let exp2 = exp(exponent2);

    // Cost formula: (P0 / k) * (e^(k * (currentSupply + tokensToBuy)) - e^(k * currentSupply))
    // We use (P0 * 10^6) / k to keep the division safe from zero
    let cost = ((INITIAL_PRICE as f64) * ((exp1 - exp2))) / K;  // Adjust for k scaling without dividing by zero
    return cost as u64;
}

pub fn exp(x: f64) -> f64 {
  let mut sum:f64 = 1.0;
  let mut term:f64 = 1.0;
  let x_power = x;
  for i in 1..20 {  // Increase iterations for better accuracy
    term = (term * x_power) / (i as f64);  // x^i / i!
    sum = sum + term;
  }
  return sum;
}

#[cfg(test)]
mod tests {
  use crate::utils::cost::*;

  #[test]
  fn test_exp() {
    //decimal = 6
    let result = exp(1.0); // e^1
    assert_eq!(result, 2.7182818284590455); //e^1
    let result = exp(0.0); // e^0 = 1
    assert_eq!(result, 1.0); //e^0
    let result = exp(2.0); // e^2 = 1
    assert_eq!(result, 7.3890560989301735); //e^2
  }
  #[test]
  fn test_cost() {
    let current_supply = 0;
    let tokens_to_buy = 100 * 1_000_000; // 100 token
    let result = calculate_cost(current_supply, tokens_to_buy, 6);
    assert_eq!(result, 800200); // 0.0008002 SOL
    
    let tokens_to_buy = 1000 * 1_000_000; // 1000 token
    let result = calculate_cost(current_supply, tokens_to_buy, 6);
    assert_eq!(result, 8020033); // 0.008020033 SOL

    let tokens_to_buy = 10000 * 1_000_000; // 10000 token
    let result = calculate_cost(current_supply, tokens_to_buy, 6);
    assert_eq!(result, 82033754); // 0.082033754 SOL

    let tokens_to_buy = 50000 * 1_000_000; // 50000 token
    let result = calculate_cost(current_supply, tokens_to_buy, 6);
    assert_eq!(result, 454440666); // 0.454440666 SOL

    let tokens_to_buy = 100_000 * 1_000_000; // 100000 token
    let result = calculate_cost(current_supply, tokens_to_buy, 6);
    assert_eq!(result, 1037954033); //1.037954033 SOL

    let tokens_to_buy = 500_000 * 1_000_000; // 500000 token
    let result = calculate_cost(current_supply, tokens_to_buy, 6);
    assert_eq!(result, 17891990337); //17.891990337 SOL
    
    let tokens_to_buy = 800_000 * 1_000_000; // 800000 token  (Max token)
    let result = calculate_cost(current_supply, tokens_to_buy, 6);
    assert_eq!(result, 85757039161); //85.757039161 SOL
  }
}