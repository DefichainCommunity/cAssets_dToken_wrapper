use dioxus::prelude::*;
use std::ops::{Add, Div, Mul, Sub};
use alloy::primitives::{utils::{format_units, parse_units},U256};
use crate::{
    config::{get_config_entry, ConfigEntry},
    metamask::uniswap_v2::{get_uniswap_v2_pairs, V2PairInfo},
    wallet_context::use_wallet,
    token::{TokenInfo, TokenType},
};

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
            pools.router_address.set(get_config_entry(info.chain_id, &ConfigEntry::VanillaV2Router).to_string());
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

pub fn is_zero_or_empty(v: Option<&str>) -> bool {
    match v {
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
    include_pool_tokens : &bool,
    include_pool : &bool,
    chain_id : u32,
) -> Vec<TokenInfo> {
    let address = if let Some(a) = selected_a{
         a.address.clone()
    }else{
        "".to_string()
    };
    let mut seen = std::collections::HashSet::new();
    let mut out = vec![];

    let native_symbol = get_config_entry(chain_id, &ConfigEntry::Native).to_string();
    let wrapped_native_address = get_config_entry(chain_id, &ConfigEntry::WrappedNativeAddress).to_string();

    for p in pairs {
        if *zero_liquid || (!is_zero_or_empty(p.reserve0.as_deref()) && !is_zero_or_empty(p.reserve1.as_deref())){
            if *include_pool{
                let pool_token = TokenInfo {
                    symbol: format!("{}-{}", p.symbol0.clone().unwrap_or("???".into()), p.symbol1.clone().unwrap_or("???".into())),
                    address: p.pair_address.clone(),
                    decimals: 18,
                    token_type: TokenType::CAsset,
                };
                if seen.insert(pool_token.address.clone()) {
                    out.push(pool_token);
                }
            }
            if *include_pool_tokens{
                // Check if Token A is token0
                if p.token0 == address || address.is_empty() {
                    let token_b = TokenInfo {
                        symbol: p.symbol1.clone().unwrap_or("???".into()),
                        address: p.token1.clone(),
                        decimals: p.decimals1.unwrap_or(18),
                        token_type: TokenType::CAsset,

                    };
                    if seen.insert(token_b.address.clone()) {
                        if token_b.address.to_lowercase() == wrapped_native_address.to_lowercase(){
                            let native = TokenInfo {
                                symbol: native_symbol.to_string(),
                                address: token_b.address.clone(),
                                decimals: token_b.decimals,
                                token_type: TokenType::Native,
                            };
                            out.push(native);
                        }
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
                        if token_b.address.to_lowercase() == wrapped_native_address.to_lowercase(){
                            let native = TokenInfo {
                                symbol: native_symbol.to_string(),
                                address: token_b.address.clone(),
                                decimals: token_b.decimals,
                                token_type: TokenType::Native,
                            };
                            out.push(native);
                        }
                        out.push(token_b);
                    }
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

pub fn as_decimal(
    amount_in: &String,
    decimals: u8,
) -> String{

    if let Ok(amount_u256) = parse_units(amount_in,0) &&
        let Ok(amount_decimal) =  format_units(amount_u256, decimals)
    {
        return amount_decimal;
    }

    "0".to_string()
}

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

pub fn get_ratio(
    reserves0: String,
    reserves1: String
) -> String {
    if let Ok(reserve0) = parse_units(&reserves0, 0) &&
        let Ok(reserve1) = parse_units(&reserves1, 0)
    {
        if reserve0.get_absolute() > 0{
            return format_units(U256::from(100_000_000) * reserve1.get_absolute() / reserve0.get_absolute(), 8).unwrap_or("0".to_string())
        }
    }
    "0".to_string()
}

pub fn calc_pool_share(
    amount_a: String,
    reserves0: String,
    decimals: u8
) -> String {
    log::debug!("Triggered calc pool share");
    if let Ok(reserve0) = parse_units(&reserves0, 0) &&
        let Ok(amount_a) = parse_units(&amount_a, decimals)
    {
        log::debug!("Amount a calc pool share :{}",amount_a);
        log::debug!("Reserve a calc pool share :{}",reserve0);
        let share = if reserve0.get_absolute() > 0 || amount_a.get_absolute() > 0{
            (amount_a.get_absolute() * U256::from(1_000_000))  / (reserve0.get_absolute() +  amount_a.get_absolute())
        }else{
            U256::from(0)
        };
        log::debug!("Triggered calc pool share :{}",share);
        return format_units(share, 4).unwrap_or("0".to_string())
    }
    "0".to_string()
}


pub fn calc_price_impact(
    amount_a: f64,
    amount_b: f64,
    r0: f64,
    r1: f64,
) -> f64 {
    if r0 == 0.0 || r1 == 0.0 { return 0.0; }
    let price_before = r0 / r1;
    let price_after = (r0 + amount_a) / (r1 + amount_b);
    ((price_after - price_before) / price_before) * 100.0
}


// pub fn calc_pool_share(
//     amount_a: f64,
//     r0: f64,
// ) -> f64 {
//     if r0 == 0.0 { return 100.0; }
//     (amount_a / (r0 + amount_a)) * 100.0
// }
