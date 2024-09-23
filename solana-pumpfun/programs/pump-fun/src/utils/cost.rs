pub const K: u64 = 8 * 1_00; //0.0008
pub const INITIAL_PRICE:u64 = 300000; // 0.0003 SOL / per token
pub fn calculate_cost(current_supply: u64, tokens_to_buy: u64) -> u64 {
    // Calculate the exponent parts scaled to avoid precision loss
    let exponent1 = (K * (current_supply + tokens_to_buy)) / 1_000_000;
    let exponent2 = (K * current_supply) / 1_000_000;

    // Calculate e^(kx) using the exp function
    let exp1 = exp(exponent1);
    let exp2 = exp(exponent2);

    // Cost formula: (P0 / k) * (e^(k * (currentSupply + tokensToBuy)) - e^(k * currentSupply))
    // We use (P0 * 10^18) / k to keep the division safe from zero
    let cost = (INITIAL_PRICE * 1000_000 * (exp1 - exp2)) / K;  // Adjust for k scaling without dividing by zero
    return cost;

}

pub fn exp(x: u64) -> u64 {
  let mut sum = 1_000_000;
  let mut term = 1_000_000;
  let x_power = x;
  for i in 1..20 {  // Increase iterations for better accuracy
    term = (term * x_power) / (i * 1000_000);  // x^i / i!
    sum = sum + term;

    // Prevent overflow and unnecessary calculations
    if term < 1 {
      break;
    }
  }
  return sum;
}