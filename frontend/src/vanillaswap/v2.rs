use dioxus::prelude::*;
use std::ops::{Add, Div, Mul, Sub};
use alloy::primitives::{utils::{format_units, parse_units},U256};
use crate::metamask::uniswap_v2::{V2PairInfo, get_uniswap_v2_pairs};
use crate::wallet_context::use_wallet;
use crate::wrapper::{TokenInfo, TokenType};

#[derive(Clone)]
pub struct UniswapV2PoolContext {
    pub pairs: Signal<Vec<V2PairInfo>>,
    pub router_address: Signal<String>,
    pub is_loading: Signal<bool>,
    pub error: Signal<Option<String>>,
}

pub fn use_v2_pools() -> UniswapV2PoolContext {
    use_context::<UniswapV2PoolContext>()
}

pub fn use_sync_v2_pools() {
    let wallet = use_wallet();
    let pools = use_v2_pools();

    use_effect(move || {
        let mut pools = pools.clone();
        let info = (wallet.info)().clone();
        spawn(async move {
            log::debug!("Sync v2 pools at chain id: {}",   info.chain_id);
            pools.pairs.set(vec![]);
            if info.chain_id == 1130{ // MainNet
                pools.router_address.set("0x3E8C92491fc73390166BA00725B8F5BD734B8fba".to_string());
            }else if  info.chain_id == 1131{ // TestNet
                pools.router_address.set("0x79208eADd9FbC29116108433a38Af62D0fD83850".to_string());
            }else{
                pools.router_address.set("".to_string());
            }

            if !info.address.is_empty() && !pools.router_address.is_empty() {
                pools.is_loading.set(true);
                log::debug!("Router address {}", pools.router_address);
                match get_uniswap_v2_pairs(&(pools.router_address)()).await {
                    Ok(list) => pools.pairs.set(list),
                    Err(e) => pools.error.set(Some(e.to_string())),
                }
                pools.is_loading.set(false);

            }
        });

    });
}

pub fn is_zero_or_empty(v: &Option<String>) -> bool {
    match v.as_deref() {
        None => true,
        Some("") => true,
        Some("0") => true,
        Some("0.0") => true,
        Some(s) => {
            // also handle cases like "0.0000"
            s.trim().parse::<f64>().map(|n| n == 0.0).unwrap_or(false)
        }
    }
}

pub fn unique_pool_tokens(
    selected_a: &Option<TokenInfo>,
    pairs: &Vec<V2PairInfo>,
    zero_liquid: &bool,
) -> Vec<TokenInfo> {
    let address = if let Some(a) = selected_a{
         a.address.clone()
    }else{
        "".to_string()
    };
    let mut seen = std::collections::HashSet::new();
    let mut out = vec![];

    for p in pairs {
        if *zero_liquid || (!is_zero_or_empty(&p.reserve0) && !is_zero_or_empty(&p.reserve1)){
            // Check if Token A is token0
            if p.token0 == address || address.is_empty() {
                let token_b = TokenInfo {
                    symbol: p.symbol1.clone().unwrap_or("???".into()),
                    address: p.token1.clone(),
                    decimals: p.decimals1.unwrap_or(18),
                    token_type: TokenType::CAsset,

                };
                if seen.insert(token_b.address.clone()) {
                    out.push(token_b);
                }
            }

            // Check if Token A is token1
            if p.token1 == address || address.is_empty(){
                let token_b = TokenInfo {
                    symbol: p.symbol0.clone().unwrap_or("???".into()),
                    address: p.token0.clone(),
                    decimals: p.decimals0.unwrap_or(18),
                    token_type: TokenType::CAsset,

                };
                if seen.insert(token_b.address.clone()) {
                    out.push(token_b);
                }
            }
        }
    }
    out.sort_by(|a, b| a.symbol.cmp(&b.symbol));
    out
}

pub fn approx_amount_out(
    amount_in: &String,
    token_a: &Option<TokenInfo>,
    token_b: &Option<TokenInfo>,
    pairs: &Vec<V2PairInfo>,
) -> String{
    let mut amount_o = U256::from(0);
    let mut decimals_o = 18; // default
    if let Some(a) = token_a &&
        let Some(b) = token_b &&
        let Some(pool) = pairs.iter().find(|p| (p.token0 == a.address && p.token1 == b.address) || (p.token0 == b.address && p.token1 == a.address)) &&
        let Ok(amount_in) = parse_units(amount_in, a.decimals as u8) &&
        let Ok(reserve0) = parse_units(&pool.reserve0.clone().unwrap_or_default(), pool.decimals0.clone().unwrap_or(18) as u8) &&
        let Ok(reserve1) = parse_units(&pool.reserve1.clone().unwrap_or_default(), pool.decimals1.clone().unwrap_or(18) as u8)
    {
        decimals_o = b.decimals;
        let (reserve_in,reserve_out) = if pool.token0 == a.address{
            (reserve0,reserve1)
        }else{
            (reserve1,reserve0)
        };
        amount_o = get_amount_out(reserve_in.get_absolute(), reserve_out.get_absolute(), amount_in.get_absolute());
    };

    log::trace!("Amount out : {}", amount_o);
    format_units(amount_o, decimals_o as u8).unwrap_or("0".to_string())
}

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
