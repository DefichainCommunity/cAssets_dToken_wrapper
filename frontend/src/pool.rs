use std::collections::BTreeMap;
use dioxus::prelude::Signal;
use crate::metamask::uniswap_v2::{V2PairInfo as PoolInfo};
use crate::vanillaswap::v2::is_zero_or_empty;

pub mod liquidity;

pub fn use_filtered_pairs(
    wrapped_native: String,
    pairs: Signal<Vec<PoolInfo>>,
    balances: Signal<BTreeMap<String, String>>,
    show_zero_liq: Signal<bool>,
    show_balanced: Signal<bool>,
) -> Vec<PoolInfo> {
    let balances = balances();
    pairs()
        .iter()
        .filter(|p| {
            // Zero liquidity filter
            (show_zero_liq()
             || (!is_zero_or_empty(p.reserve0.as_deref())
                 && !is_zero_or_empty(p.reserve1.as_deref())))
                &&
            // Balanced filter
                (!show_balanced()
                 || !is_zero_or_empty(balances.get(&p.pair_address).map(String::as_str))
                 || (
                     (!is_zero_or_empty(balances.get(&p.token0).map(String::as_str))
                      || (p.token0.to_lowercase() == wrapped_native
                          && !is_zero_or_empty(balances.get("native").map(String::as_str))))
                         &&
                         (!is_zero_or_empty(balances.get(&p.token1).map(String::as_str))
                          || (p.token1.to_lowercase() == wrapped_native
                              && !is_zero_or_empty(balances.get("native").map(String::as_str))))
                 ))
        })
        .cloned()
        .collect::<Vec<PoolInfo>>()
}

pub fn get_pool(address_a: &String, address_b: &String, pools: &Vec<PoolInfo>) -> Option<PoolInfo>{
    pools.iter().find(|p| (p.token0 == *address_a &&  p.token1 == *address_b) || (p.token1 == *address_a &&  p.token0 == *address_b)).cloned()
}
