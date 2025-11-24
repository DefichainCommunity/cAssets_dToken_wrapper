use dioxus::prelude::*;
use wasm_bindgen_futures::spawn_local;
use serde::Deserialize;
use std::collections::BTreeMap;
use alloy::primitives::{utils::{format_units, parse_units},U256};
use crate::components::switch::{Switch, SwitchThumb};
use crate::{metamask::get_uniswap_v3_pool_states, wallet_context::use_wallet};
use crate::metamask::V3PoolState;

#[derive(Deserialize, Debug, Clone)]
struct PoolInfo {
    token0: String,
    symbol0: String,
    decimals0: u64,
    token1: String,
    symbol1: String,
    decimals1:u64,
    fee: u32,
    tick_spacing: i32,
    pair_address: String,
}

pub fn load_pools(chain_id: u32) -> Vec<PoolInfo> {
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
    log::debug!("Base ratio {}", base_ratio);
    match format_units(scaled_price, precision as u8) {
        Ok(s) => s,
        Err(_) => "???".into(),
    }
}

fn format_liquidity(liq: u128) -> String {
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

#[component]
pub fn PoolV3Pairs() -> Element {
    // let mut pairs = use_signal(|| Vec::<PairInfo>::new());
    let mut is_loading = use_signal(|| false);
    let mut pool_state = use_signal(|| BTreeMap::new());
    let mut error = use_signal(|| None::<String>);
    let mut show_zero_liq = use_signal(|| false);
    let mut pairs = use_signal(|| Vec::<PoolInfo>::new());
    let p = pairs.clone();
    use_effect(move || {
        let wallet = use_wallet();
        // this ensures that we react to address changes
        let _info = (wallet.info)().clone();
        let pp =  p.clone();
        spawn_local(async move {
            let info = (wallet.info)().clone();
            pairs.set(load_pools(info.chain_id));
            if !info.address.is_empty(){
                is_loading.set(true);
                let pools = pp.iter().map(|p| p.pair_address.clone()).collect();
                match get_uniswap_v3_pool_states(pools).await{
                    Ok(list) => {
                        // log::debug!("{:?}", list);
                        pool_state.set(list)
                    },
                    Err(e) => error.set(Some(e.to_string())),
                }
                is_loading.set(false);

            }
        });
    });


    rsx! {
        div { class: "p-8 mt-12 glass w-full max-w-4xl flex flex-col gap-6 items-stretch flex-col-sm",
              h2 { class: "text-3xl font-bold text-center mb-6", "V3 PoolPairs" }
              div { class: "flex items-center gap-2",
                    span { class: "text-gray-200 text-sm", "Show zero liquidity pools" }
                    Switch {
                        checked: show_zero_liq(),
                        on_checked_change: move |new_state| show_zero_liq.set(new_state),
                        SwitchThumb {}
                    }
              }
              if *is_loading.read() {
                  div { class: "text-gray-300", "Loading..." }
              }

              if let Some(err) = &*error.read() {
                  div { class: "text-red-500 mb-4", "{err}" }
              }

              // ---------- PAIRS LIST ----------
              div { class: "flex flex-col gap-3",
                    {pairs.iter().filter_map(|pair| {
                        let state = pool_state.read().get(&pair.pair_address).cloned().unwrap_or_default();
                        let liquidity = state.liquidity;
                        if  liquidity > 0_u128 || show_zero_liq(){
                            log::debug!("Pool: {}-{}", pair.symbol0, pair.symbol1);
                            let price = price_from_sqrt_price(&state.sqrt_price_x96, pair.decimals0, pair.decimals1);
                            Some(rsx!{
                                div { class: "p-4 bg-gray-900/60 border border-gray-800 rounded-xl shadow-md
                                    flex flex-col gap-2 hover:bg-gray-900 transition-colors duration-200",
                                      // ROW 1
                                      div { class: "flex items-center",
                                            // TOKEN 0
                                            div { class: "flex items-center gap-2",
                                                  div { class: "text-gray-200 font-semibold text-lg",
                                                        "{ pair.symbol0.clone()}"
                                                  }
                                            }

                                            // Slash between tokens
                                            div { class: "text-gray-500 font-bold text-xl", "/" }

                                            // TOKEN 1
                                            div { class: "flex items-center gap-2",
                                                  div { class: "text-gray-200 font-semibold text-lg",
                                                        "{ pair.symbol1.clone() }"
                                                  }
                                            }

                                            // ADDRESS RIGHT SIDE
                                            div { class: "ml-auto text-xs text-gray-500 tracking-wide",
                                                  "{ pair.pair_address }"
                                            }
                                      }
                                      // ROW 2
                                      div { class: "flex items-center text-xs font-bold text-gray-500",
                                            div { class: "w-1/5",
                                                  "Fee: { (pair.fee as f64) / 10000.0 }%"
                                            }

                                            // COLUMN 2 — LIQUIDITY
                                            div { class: "w-2/5 text-center",
                                                  "Liquidity: {format_liquidity(liquidity)}"
                                            }

                                            // COLUMN 3 — PRICE
                                            div { class: "w-2/5 text-right",
                                                  "Price token1/token0: {price}"
                                            }
                                      }
                                }
                            })
                        }else{None}
                    }
                    )}
              }
        }
    }

}
