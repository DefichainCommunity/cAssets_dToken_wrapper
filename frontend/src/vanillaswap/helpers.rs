use std::ops::{Add, Div, Mul, Sub};

use alloy::primitives::U256;

// /// Displays the token amount in a human-readable format
// pub fn display_token(value: U256) -> String {
//     format!("{:.18}", value.low_u128() as f64 / 1_000_000_000_000_000_000.0)
// }

// https://github.com/alloy-rs/examples/blob/main/examples/advanced/examples/uniswap_u256/helpers/alloy.rs

/// Get amount out for Uniswap V2
pub fn get_amount_out(reserve_in: U256, reserve_out: U256, amount_in: U256) -> U256 {
    let amount_in_with_fee = amount_in * get_uniswappy_fee();
    let numerator = amount_in_with_fee * reserve_out;
    let denominator = reserve_in * U256::from(1000) + amount_in_with_fee;
    if denominator.is_zero(){
        return U256::from(0);
    }
    numerator / denominator
}

/// Get amount in for Uniswap V2
pub fn get_amount_in(
    reserves00: U256,
    reserves01: U256,
    is_weth0: bool,
    reserves10: U256,
    reserves11: U256,
) -> U256 {
    let numerator = get_numerator(reserves00, reserves01, is_weth0, reserves10, reserves11);

    let denominator = get_denominator(reserves00, reserves01, is_weth0, reserves10, reserves11);
    if denominator.is_zero(){
        return U256::from(0);
    }
    numerator * U256::from(1000) / denominator
}

fn sqrt(input: U256) -> U256 {
    if input == U256::ZERO {
        return U256::ZERO;
    }

    let mut z = (input + U256::from(1)) / U256::from(2);
    let mut y = input;
    while z < y {
        y = z;
        z = (input / z + z) / U256::from(2);
    }
    y
}

fn get_numerator(
    reserves00: U256,
    reserves01: U256,
    is_weth0: bool,
    reserves10: U256,
    reserves11: U256,
) -> U256 {
    if is_weth0 {
        let presqrt = get_uniswappy_fee()
            .mul(get_uniswappy_fee())
            .mul(reserves01)
            .mul(reserves10)
            .div(reserves11)
            .div(reserves00);
        sqrt(presqrt).sub(U256::from(1000)).mul(reserves11).mul(reserves00)
    } else {
        let presqrt = get_uniswappy_fee()
            .mul(get_uniswappy_fee())
            .mul(reserves00)
            .mul(reserves11)
            .div(reserves10)
            .div(reserves01);
        (sqrt(presqrt)).sub(U256::from(1000)).mul(reserves10).mul(reserves01)
    }
}

fn get_denominator(
    reserves00: U256,
    reserves01: U256,
    is_weth0: bool,
    reserves10: U256,
    reserves11: U256,
) -> U256 {
    if is_weth0 {
        get_uniswappy_fee()
            .mul(reserves11)
            .mul(U256::from(1000))
            .add(get_uniswappy_fee().mul(get_uniswappy_fee()).mul(reserves01))
    } else {
        get_uniswappy_fee()
            .mul(reserves10)
            .mul(U256::from(1000))
            .add(get_uniswappy_fee().mul(get_uniswappy_fee()).mul(reserves00))
    }
}

fn get_uniswappy_fee() -> U256 {
    U256::from(997)
}
