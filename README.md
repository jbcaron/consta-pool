# ConstaPool

**ConstaPool** is a Rust library implementing a constant product liquidity pool model, based on the formula `a * b = k`. This model is widely used in decentralized finance (DeFi) for automated market making, allowing for efficient and decentralized token swaps.

## Features

- **Constant Product Formula**: Implements the liquidity pool mechanism where the product of reserves remains constant during trades.
- **Slippage Management**: Handles slippage control to prevent significant price deviations during trades.
- **Simulations**: Provides functions to simulate trades without executing them, enabling users to estimate costs and returns.
- **Error Handling**: Comprehensive error handling using the `thiserror` crate, covering common pool errors such as insufficient funds, overflow, and invalid amounts.

## Usage

Here’s a basic example of how to use ConstaPool:

```rust
use consta_pool::{LiquidityPool, PoolError};

fn main() -> Result<(), PoolError> {
    // Create a new liquidity pool with specified reserves
    let mut pool = LiquidityPool::new(30 * 10u64.pow(9), 1_000_000_000 * 10u64.pow(6))?;

    // Buy tokens from the pool
    let token_amount = 1_000_000 * 10u64.pow(6); // Buying 1,000,000 tokens
    let native_spent = pool.buy_tokens(token_amount, None)?;
    println!("Spent {} native to buy {} tokens", native_spent, token_amount);

    // Sell tokens back to the pool
    let native_received = pool.sell_tokens(token_amount, None)?;
    println!("Sold {} tokens to receive {} native", token_amount, native_received);

    Ok(())
}
```

## API Overview

- **Creating a Pool:** Initialize a liquidity pool with specified native and token reserves.
- **Buying Tokens:** Buy tokens from the pool with buy_tokens(token_amount, max_native), where max_native is an optional slippage limit.
- **Selling Tokens:** Sell tokens back to the pool with sell_tokens(token_amount, min_native), where min_native is an optional slippage limit.
- **Simulations:** Use simulate_buy_tokens and simulate_sell_tokens to estimate the costs or returns of trades without executing them.
- **Error Handling:** ConstaPool uses the thiserror crate to provide detailed error handling, including:
  - **SlippageExceeded:** Indicates that the trade exceeded the acceptable slippage limit.
  - **InsufficientPoolFunds:** Indicates insufficient funds in the pool to complete the trade.
  - **InvalidAmount:** Indicates an invalid amount provided, such as zero or negative values.
  - **Overflow:** Indicates arithmetic overflow, usually when dealing with large numbers.

## License

This project is licensed under the terms of the [MIT License](LICENSE).
