use std::cmp::Ordering;

type Result<T> = std::result::Result<T, PoolError>;

#[derive(Debug, Clone)]
pub struct LiquidityPool {
    initial_token_reserve: u64,
    native_reserve: u64,
    token_reserve: u64,
    constant_product: u128,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum PoolError {
    #[error("Slippage too high")]
    SlippageExceeded,
    #[error("Invalid funds in the pool")]
    InsufficientPoolFunds,
    #[error("Invalid amount")]
    InvalidAmount,
    #[error("Overflow")]
    Overflow,
}

impl LiquidityPool {
    pub fn new(native_reserve: u64, token_reserve: u64) -> Result<Self> {
        if native_reserve == 0 || token_reserve == 0 {
            return Err(PoolError::InvalidAmount);
        }
        let constant_product = native_reserve as u128 * token_reserve as u128;
        Ok(Self {
            initial_token_reserve: token_reserve,
            native_reserve,
            token_reserve,
            constant_product,
        })
    }

    pub fn get_native_reserve(&self) -> u64 {
        self.native_reserve
    }
    pub fn get_token_reserve(&self) -> u64 {
        self.token_reserve
    }

    pub fn get_constant_product(&self) -> u128 {
        self.constant_product
    }

    /// Returns the current market price of tokens in terms of native currency.
    pub fn market_price(&self) -> f64 {
        self.native_reserve as f64 / self.initial_token_reserve as f64
    }

    /// Buys `token_amount` tokens from the pool, checking if the native currency spent does not exceed `max_native`.
    pub fn buy(&mut self, token_amount: u64, max_native: Option<u64>) -> Result<u64> {
        if token_amount == 0 {
            return Err(PoolError::InvalidAmount);
        }
        let new_token_reserve = self
            .token_reserve
            .checked_sub(token_amount)
            .ok_or(PoolError::InsufficientPoolFunds)?;
        let new_native_reserve = self
            .constant_product
            .checked_div(new_token_reserve as u128)
            .ok_or(PoolError::Overflow)? as u64;
        let native_sold = new_native_reserve - self.native_reserve;
        if let Some(max_native) = max_native {
            if native_sold > max_native {
                return Err(PoolError::SlippageExceeded);
            }
        }
        self.native_reserve = new_native_reserve;
        self.token_reserve = new_token_reserve;
        Ok(native_sold)
    }

    /// Sells `token_amount` tokens to the pool, checking if the native currency received is at least `min_native`.
    pub fn sell(&mut self, token_amount: u64, min_native: Option<u64>) -> Result<u64> {
        if token_amount == 0 {
            return Err(PoolError::InvalidAmount);
        }
        if token_amount > self.token_reserve {
            return Err(PoolError::InsufficientPoolFunds);
        }
        let new_token_reserve = self
            .token_reserve
            .checked_add(token_amount)
            .ok_or(PoolError::Overflow)?;
        let new_native_reserve = self
            .constant_product
            .checked_div(new_token_reserve as u128)
            .ok_or(PoolError::Overflow)? as u64;
        let native_bought = self.native_reserve - new_native_reserve;
        if let Some(min_native) = min_native {
            if native_bought < min_native {
                return Err(PoolError::SlippageExceeded);
            }
        }
        self.native_reserve = new_native_reserve;
        self.token_reserve = new_token_reserve;
        Ok(native_bought)
    }

    /// Simulates buying `token_amount` tokens and calculates the native currency that would be spent.
    pub fn simulate_buy(&self, token_amount: u64, min_native: Option<u64>) -> Result<u64> {
        if token_amount == 0 {
            return Err(PoolError::InvalidAmount);
        }
        let new_token_reserve = self
            .token_reserve
            .checked_sub(token_amount)
            .ok_or(PoolError::InsufficientPoolFunds)?;
        let new_native_reserve = self
            .constant_product
            .checked_div(new_token_reserve as u128)
            .ok_or(PoolError::Overflow)? as u64;
        let native_sold = new_native_reserve - self.native_reserve;
        if let Some(min_native) = min_native {
            if native_sold < min_native {
                return Err(PoolError::SlippageExceeded);
            }
        }
        Ok(native_sold)
    }

    /// Simulates selling `token_amount` tokens and calculates the native currency that would be received.
    pub fn simulate_sell(&self, token_amount: u64, max_native: Option<u64>) -> Result<u64> {
        if token_amount == 0 {
            return Err(PoolError::InvalidAmount);
        }
        if token_amount > self.token_reserve {
            return Err(PoolError::InsufficientPoolFunds);
        }
        let new_token_reserve = self
            .token_reserve
            .checked_add(token_amount)
            .ok_or(PoolError::Overflow)?;
        let new_native_reserve = self
            .constant_product
            .checked_div(new_token_reserve as u128)
            .ok_or(PoolError::Overflow)? as u64;
        let native_sold = self.native_reserve - new_native_reserve;
        if let Some(max_native) = max_native {
            if native_sold > max_native {
                return Err(PoolError::SlippageExceeded);
            }
        }
        Ok(native_sold)
    }

    /// Calculates the amount of tokens that would be received for spending a specific amount of native currency.
    pub fn calculate_tokens_received(&self, native_amount: u64) -> Result<u64> {
        if native_amount == 0 {
            return Err(PoolError::InvalidAmount);
        }
        let new_native_reserve = self.native_reserve + native_amount;
        let new_token_reserve = self
            .constant_product
            .checked_div(new_native_reserve as u128)
            .ok_or(PoolError::Overflow)? as u64;
        Ok(self.token_reserve - new_token_reserve)
    }

    /// Buys tokens using a specified amount of native currency.
    pub fn buy_tokens_with_native(&mut self, native_amount: u64) -> Result<u64> {
        if native_amount == 0 {
            return Err(PoolError::InvalidAmount);
        }
        let token_amount = self.calculate_tokens_received(native_amount)?;
        self.buy(token_amount, None)?;
        Ok(token_amount)
    }

    pub fn calculate_price_impact(&self, token_amount: u64) -> f64 {
        let initial_price = self.market_price();
        let new_token_reserve = self.token_reserve - token_amount;
        let new_native_reserve = self.constant_product / new_token_reserve as u128;
        let new_price = new_native_reserve as f64 / new_token_reserve as f64;
        (new_price - initial_price) / initial_price
    }

    /// Calculates the number of additional tokens required to reach a desired native currency amount.
    pub fn calculate_additional_tokens_for_desired_native(
        &mut self,
        sell_tokens: u64,
        desired_native: u64,
    ) -> Result<u64> {
        if sell_tokens == 0 || desired_native == 0 {
            return Err(PoolError::InvalidAmount);
        }

        let mut low = 0u64;
        let mut high = self.token_reserve;
        let mut best_guess = high;

        // Binary search to find the best amount of tokens to buy
        while low <= high {
            let mid = low + (high - low) / 2;

            // Clone the pool to simulate the impact of buying and selling without affecting the actual pool
            let mut temp_pool = self.clone();

            // Attempt to buy `mid` tokens
            temp_pool.buy(mid, None)?;

            // Simulate selling `sell_tokens` tokens to see how much native currency would be received
            let native_received = temp_pool.simulate_sell(sell_tokens, None)?;

            match native_received.cmp(&desired_native) {
                Ordering::Equal => return Ok(mid),
                Ordering::Less => {
                    // Not enough native currency received, increase the number of tokens to buy
                    low = mid.checked_add(1).ok_or(PoolError::Overflow)?;
                }
                Ordering::Greater => {
                    // Too much native currency received, decrease the number of tokens to buy
                    best_guess = mid;
                    high = mid.checked_sub(1).ok_or(PoolError::Overflow)?;
                }
            }
            // Exit the loop when the search range is narrowed sufficiently
            if high - low <= 1 {
                break;
            }
        }

        Ok(best_guess)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl LiquidityPool {
        /// Checks the integrity of the pool by comparing the constant product after operations.
        fn check_pool_integrity(&self) -> Result<f64> {
            if self.native_reserve == 0 || self.token_reserve == 0 {
                return Err(PoolError::InvalidAmount);
            }
            let k = self.native_reserve as u128 * self.token_reserve as u128;
            Ok((k as f64 - self.constant_product as f64) / self.constant_product as f64)
        }
    }

    impl Default for LiquidityPool {
        fn default() -> Self {
            let native_reserve = 10u64.pow(9);
            let token_reserve = 1_000_000_000 * 10u64.pow(6);
            Self::new(native_reserve, token_reserve).unwrap()
        }
    }

    #[test]
    fn test_buy() {
        let mut pool = LiquidityPool::default();
        let native_reserve = pool.get_native_reserve();
        let token_reserve = pool.get_token_reserve();
        let token_amount = 1_000_000 * 10u64.pow(6);
        let native = pool.buy(token_amount, None).unwrap();
        println!(
            "Get {} tokens by paying {:.6} NATIVE",
            token_amount / 10u64.pow(6),
            native as f64 / 10u64.pow(9) as f64
        );
        assert_eq!(pool.get_native_reserve(), native_reserve + native);
        assert_eq!(pool.get_token_reserve(), token_reserve - token_amount);
        println!("Integrity: {}", pool.check_pool_integrity().unwrap());
        assert!(pool.check_pool_integrity().unwrap().abs() < 0.000000001);
    }

    #[test]
    fn test_sell() {
        let mut pool = LiquidityPool::default();
        let native_reserve = pool.get_native_reserve();
        let token_reserve = pool.get_token_reserve();
        let token_amount = 1_000_000 * 10u64.pow(6);
        let native = pool.sell(token_amount, None).unwrap();
        println!(
            "Sell {} tokens and get {:.6} NATIVE",
            token_amount / 10u64.pow(6),
            native as f64 / 10u64.pow(9) as f64
        );
        assert_eq!(pool.get_native_reserve(), native_reserve - native);
        assert_eq!(pool.get_token_reserve(), token_reserve + token_amount);
        println!("Integrity: {}", pool.check_pool_integrity().unwrap());
        assert!(pool.check_pool_integrity().unwrap().abs() < 0.000000001);
    }

    #[test]
    fn test_simulate_buy() {
        let pool = LiquidityPool::default();
        let token_amount = 1_000_000 * 10u64.pow(6);
        let native = pool.simulate_buy(token_amount, None).unwrap();
        let new_k = (pool.get_native_reserve() + native) as u128
            * (pool.get_token_reserve() - token_amount) as u128;
        let difference = (new_k as f64 - pool.get_constant_product() as f64)
            / pool.get_constant_product() as f64;
        assert!(difference.abs() < 0.000000001);
    }

    #[test]
    fn test_simulate_sell() {
        let pool = LiquidityPool::default();
        let token_amount = 1_000_000 * 10u64.pow(6);
        let native = pool.simulate_sell(token_amount, None).unwrap();
        println!(
            "Simulate sell {} tokens and get {:.6} NATIVE",
            token_amount / 10u64.pow(6),
            native as f64 / 10u64.pow(9) as f64
        );
        let new_k = (pool.get_native_reserve() - native) as u128
            * (pool.get_token_reserve() + token_amount) as u128;
        let difference = (new_k as f64 - pool.get_constant_product() as f64)
            / pool.get_constant_product() as f64;
        println!("Integrity: {}", difference);
        assert!(difference.abs() < 0.000000001);
    }

    #[test]
    fn test_calculate_tokens_received() {
        let mut pool = LiquidityPool::default();
        let native_amount = 10u64.pow(9);
        let token_amount = pool.calculate_tokens_received(native_amount).unwrap();
        println!(
            "Should get {:.6} tokens by paying {:.6} NATIVE",
            token_amount as f64 / 10u64.pow(6) as f64,
            native_amount as f64 / 10u64.pow(9) as f64
        );
        let native = pool.buy(token_amount, None).unwrap();
        assert_eq!(native, native_amount);
    }

    #[test]
    fn test_buy_tokens_with_native() {
        let mut pool = LiquidityPool::default();
        let native_amount = 10u64.pow(9);
        let initial_native_reserve = pool.get_native_reserve();
        let token_amount = pool.buy_tokens_with_native(native_amount).unwrap();
        println!(
            "Get {} tokens by paying {:.6} NATIVE",
            token_amount / 10u64.pow(6),
            native_amount as f64 / 10u64.pow(9) as f64
        );
        assert_eq!(
            pool.get_native_reserve(),
            initial_native_reserve + native_amount
        );
    }

    #[test]
    fn test_buy_invalid_slippage() {
        let mut pool = LiquidityPool::default();
        let token_amount = 1_000_000 * 10u64.pow(6);
        let native_cost = pool.simulate_buy(token_amount, None).unwrap();
        let result = pool.buy(token_amount, Some(native_cost - 1));
        assert_eq!(result, Err(PoolError::SlippageExceeded));
    }

    #[test]
    fn test_sell_invalid_slippage() {
        let mut pool = LiquidityPool::default();
        let token_amount = 1_000_000 * 10u64.pow(6);
        let native_gain = pool.simulate_sell(token_amount, None).unwrap();
        let result = pool.sell(token_amount, Some(native_gain + 1));
        assert_eq!(result, Err(PoolError::SlippageExceeded));
    }

    #[test]
    fn test_calculate_missing_tokens() {
        let mut pool = LiquidityPool::default();
        let tokens_to_buy = 50_000_000 * 10u64.pow(6);
        let native_spent = pool.buy(tokens_to_buy, None).unwrap();

        let additional_native_needed = native_spent + 10u64.pow(9);
        let missing_tokens = pool
            .calculate_additional_tokens_for_desired_native(tokens_to_buy, additional_native_needed)
            .unwrap();

        pool.buy(missing_tokens, None).unwrap();

        let native_received = pool.sell(tokens_to_buy, None).unwrap();

        assert!(native_received >= additional_native_needed);
        assert!(native_received <= additional_native_needed + 1);
    }

    #[test]
    fn test_many_operations() {
        let mut pool = LiquidityPool::default();
        let token_buy_amouts = [1_000_000, 10_000_000, 3_000_000, 3_000_000, 1_000_000];
        let token_sell_amouts = [1_000_000, 1_000_000, 2_000_000, 1_000_000, 5_000_000];
        for i in 0..5 {
            let token_amount = token_buy_amouts[i] * 10u64.pow(6);
            let _ = pool.buy(token_amount, None).unwrap();
            let token_amount = token_sell_amouts[i] * 10u64.pow(6);
            let _ = pool.sell(token_amount, None).unwrap();
        }

        assert!(pool.check_pool_integrity().unwrap().abs() < 0.000000001);
    }
}
