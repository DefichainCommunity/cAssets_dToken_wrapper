use std::collections::BTreeMap;
use alloy::primitives::{utils::{format_units, parse_units},U256};
use dioxus::prelude::*;
use serde::Deserialize;
use crate::{
    config::{get_config_entry, ConfigEntry},
    metamask::uniswap_v3::{V3PoolState, get_uniswap_v3_pool_states},
    wallet_context::use_wallet,
    token::{TokenInfo, TokenType},
};


#[derive(Clone)]
pub struct UniswapV3PoolContext {
    pub pairs: Signal<Vec<V3PoolInfo>>,
    pub pool_state: Signal<BTreeMap<String, V3PoolState>>,
    pub router_address: Signal<String>,
    pub is_loading: Signal<bool>,
    pub error: Signal<Option<String>>,
}

pub fn use_v3_pools() -> UniswapV3PoolContext {
    use_context::<UniswapV3PoolContext>()
}

pub fn use_sync_v3_pools() {
    let wallet = use_wallet();
    let pools = use_v3_pools();

    use_effect(move || {
        let mut pools = pools.clone();
        let info = (wallet.info)().clone();
        spawn(async move {
            log::debug!("Sync v3 pools at chain id: {}",   info.chain_id);
            // Always refresh the list of pools for this chain
            pools.pairs.set(load_pools(info.chain_id));
            pools.router_address.set(get_config_entry(info.chain_id, &ConfigEntry::VanillaV3Router).to_string());
            // Only fetch pool state if connected
            if !info.address.is_empty() && !pools.router_address.is_empty(){
                pools.is_loading.set(true);

                let pool_addresses =
                    pools.pairs.read().iter()
                        .map(|p| p.pair_address.clone())
                        .collect::<Vec<_>>();

                match get_uniswap_v3_pool_states(pool_addresses).await {
                    Ok(list) => pools.pool_state.set(list),
                    Err(e) => pools.error.set(Some(e.to_string())),
                }

                pools.is_loading.set(false);
            }
        });

    });
}

pub fn unique_pool_tokens(
    selected_a: &Option<TokenInfo>,
    pairs: &Vec<V3PoolInfo>,
    state: &BTreeMap<String, V3PoolState>,
    zero_liquid: &bool,
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
        if let Some(state) = state.get(&p.pair_address){
            if *zero_liquid || state.liquidity > 0_u128{
                // Check if Token A is token0
                if p.token0 == address || address.is_empty() {
                    let token_b = TokenInfo {
                        symbol: p.symbol1.clone(),
                        address: p.token1.clone(),
                        decimals: p.decimals1,
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
                        symbol: p.symbol0.clone(),
                        address: p.token0.clone(),
                        decimals: p.decimals0,
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

pub fn inverse_price(price: U256, decimals: u8) -> U256 {
    let scale = U256::from(10).pow(U256::from(decimals));
    (scale * scale) / price
}

pub fn approx_amount_out(
    amount_in: &String,
    token_a: &Option<TokenInfo>,
    token_b: &Option<TokenInfo>,
    pairs: &Vec<V3PoolInfo>,
    pool_state : &BTreeMap<String,V3PoolState>,
) -> String{
    let mut amount_o = U256::from(0);
    let decimals_o = 18; // default
    if let Some(a) = token_a &&
        let Some(b) = token_b &&
        let Ok(amount_in) = parse_units(amount_in, a.decimals as u8) &&
        let Some(pool) = pairs.iter().find(|p| (p.token0 == a.address && p.token1 == b.address) || (p.token0 == b.address && p.token1 == a.address)) &&
        let Some(pool_state) = pool_state.get(&pool.pair_address)
    {
        let price = price_from_sqrt_price(&pool_state.sqrt_price_x96, a.decimals, b.decimals);
        log::trace!("Price from sqrt {}", price);
        if let Ok(price) = parse_units(&price, 18){
            let mut price = price.get_absolute();
            log::trace!("Price {}", price);
            if pool.token0 == b.address{

                price = inverse_price(price, decimals_o.into());
                log::trace!("Price invert {}", price);
            }
            amount_o = (price * amount_in.get_absolute())/U256::from(10).pow(U256::from(decimals_o));
        }
    }
    log::trace!("Amount out : {}", amount_o);
    format_units(amount_o, decimals_o as u8).unwrap_or("0".to_string())
}


#[derive(Deserialize, Debug, Clone)]
pub struct V3PoolInfo {
    pub token0: String,
    pub symbol0: String,
    pub decimals0: u64,
    pub token1: String,
    pub symbol1: String,
    pub decimals1:u64,
    pub fee: u32,
    pub tick_spacing: i32,
    pub pair_address: String,
}

pub fn load_pools(chain_id: u32) -> Vec<V3PoolInfo> {
    if chain_id == 1130{
        return serde_json::from_str(include_str!("../../assets/pools.json")).unwrap();
    }
    vec![]
}


// sqrtPriceX96 → token1/token0 price
pub fn price_from_sqrt_price(
    sqrt_price_x96: &str,
    decimals0: u64,
    decimals1: u64
) -> String {
    // Parse sqrtPriceX96 as U256
    let Ok(sp) = parse_units(&sqrt_price_x96, 0) else {
        return "???".into();
    };
    // 10^x using Alloy .pow()
    let pow10 = |x: usize| U256::from(10).pow(U256::from(x));

    let precision : u8 = 18;
    let numerator = sp.get_absolute() * sp.get_absolute();
    // we add 18 fixed point precision positions
    let numerator = numerator * pow10(precision.into());
    let denom = U256::from(1) << 192;
    let base_ratio = numerator / denom;
    // Apply decimal correction
    let decimal_adjust = (decimals0 as i64) - (decimals1 as i64);


    // Convert ratio to 18-decimal fixed point
    let scaled_price = if decimal_adjust >= 0 {
        base_ratio * pow10(decimal_adjust as usize)
    } else {
        base_ratio / pow10((-decimal_adjust) as usize)
    };
    match format_units(scaled_price, precision as u8) {
        Ok(s) => s,
        Err(_) => "???".into(),
    }
}

pub fn format_liquidity(liq: u128) -> String {
    if liq >= 1_000_000_000 {
        format!("{:.0} B", (liq as f64) / 1_000_000_000_f64)
    } else if liq >= 1_000_000 {
        format!("{:.0} M", (liq as f64) / 1_000_000_f64)
    } else if liq >= 1_000 {
        format!("{:.0} K", (liq as f64) / 1_000_f64)
    } else {
        liq.to_string()
    }
}
