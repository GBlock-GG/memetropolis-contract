use crate::*;

// use crate::utils::convert_to_float;

pub const K: f64 = 0.000005;
pub const INITIAL_PRICE: u64 = 8000; // 0.000008  SOL / per token
pub fn calculate_cost(current_supply: u64, tokens_to_buy: u64, decimals: u8) -> u64 {
    // // Calculate the exponent parts scaled to avoid precision loss
    let exponent1 = K * convert_to_float(current_supply + tokens_to_buy, decimals);
    let exponent2 = K * convert_to_float(current_supply, decimals);

    // // Calculate e^(kx) using the exp function
    let exp1 = exponent1.exp();
    let exp2 = exponent2.exp();

    // Cost formula: (P0 / k) * (e^(k * (currentSupply + tokensToBuy)) - e^(k * currentSupply))
    // We use (P0 * 10^6) / k to keep the division safe from zero
    let cost = ((INITIAL_PRICE as f64) * (exp1 - exp2)) / K; // Adjust for k scaling without dividing by zero
    return cost as u64;
}

pub fn calculate_token_amount(current_supply: u64, sol_amount: u64, decimals: u8) -> u64 {
    // k * currentSupply
    let exponent = K * convert_to_float(current_supply, decimals);

    // Calculate e^(k * currentSupply)
    let exp1 = exponent.exp();
    // Calculate (solAmount * K) / INIT_PRICE
    let num = ((sol_amount as f64) * K) / (INITIAL_PRICE as f64);
    // Calculate ln((solAmount * K) / INIT_PRICE + e^(k * currentSupply))
    let ln = (num + exp1).ln();
    // formula: (ln((ethAmount * K) / INIT_PRICE + e^(k * currentSupply)) / K) - currentSupply
    let token_amount = convert_from_float(ln / K , decimals) - current_supply;
    return token_amount;
}

// pub fn exp(x: f64) -> f64 {
//     let mut sum: f64 = 1.0;
//     let mut term: f64 = 1.0;
//     let x_power = x;
//     for i in 1..20 {
//         // Increase iterations for better accuracy
//         term = (term * x_power) / (i as f64); // x^i / i!
//         sum = sum + term;
//     }
//     return sum;
// }

// pub fn ln(x: f64) -> Result<f64> {
//     require!(x > 0.0, PumpFunError::InvalidInput);
//     let log2_result = x.ln()
//     Ok(0 as f64)
// }
// pub fn log_2(x: f64) -> Result<f64> {

// }
#[cfg(test)]
mod tests {
    use crate::utils::cost::*;

    #[test]
    fn test_exp() {
        //decimal = 6
        let mut x = 1.0 as f64;
        let result = x.exp(); // e^1
        assert_eq!(result, 2.718281828459045); //e^1
        x = 0.0;
        let result = x.exp(); // e^0 = 1
        assert_eq!(result, 1.0);
        x = 2.0;
        let result = x.exp(); // e^2 = 1
        assert_eq!(result, 7.38905609893065); //e^2
    }
    #[test]
    fn test_ln(){
        let mut x = 1.0 as f64;
        let mut result = x.ln();
        assert_eq!(result, 0.0); //e^0 = 1

        x = 2.718281828459045 as f64; //e
        result = x.ln(); //e^1 = e
        assert_eq!(result, 1.0);
        x = 7.38905609893065 as f64;
        result = x.ln();
        assert_eq!(result, 2.0); //e^2 = 7.38905609893065
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
        assert_eq!(result, 85757040053); //85.757040053 SOL
    }

    #[test]
    fn test_calculate_token_amount() {
        let current_supply = 0;
        let sol_amount = 800200; // 0.0008002 SOL
        let result = calculate_token_amount(current_supply, sol_amount, 6);
        assert_eq!(result, 99999995); // 99.999995 TOKEN

        let sol_amount = 8020033u64; // 0.008020033 SOL
        let result = calculate_token_amount(current_supply, sol_amount, 6);
        assert_eq!(result, 999999953); // 999.999953 token

        let sol_amount = 82033754u64; // 0.082033754 SOL
        let result = calculate_token_amount(current_supply, sol_amount, 6);
        assert_eq!(result, 9999999976); // 9999.999976 token

        let sol_amount = 454440666; // 0.454440666 SOL
        let result = calculate_token_amount(current_supply, sol_amount, 6);
        assert_eq!(result, 49999999931); // 49999.999931 token

        let sol_amount = 1037954033; // 1.037954033 SOL
        let result = calculate_token_amount(current_supply, sol_amount, 6);
        assert_eq!(result, 99999999990); // 99999.999990 token

        let sol_amount = 17891990337; // 17.891990337 SOL
        let result = calculate_token_amount(current_supply, sol_amount, 6);
        assert_eq!(result, 499999999998); // 499999.999998 token

        let sol_amount = 85757040053; // 85.757040053 SOL
        let result = calculate_token_amount(current_supply, sol_amount, 6);
        assert_eq!(result, 799999999999); // 799999.999999token

    }
}
