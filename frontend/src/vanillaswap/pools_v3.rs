use dioxus::prelude::*;
use wasm_bindgen_futures::spawn_local;
use serde::Deserialize;
use std::collections::BTreeMap;
use crate::components::switch::{Switch, SwitchThumb};
use crate::vanillaswap::v3::use_v3_pools;
use crate::wallet_context::use_wallet;
use crate::metamask::uniswap_v3::get_uniswap_v3_pool_states;
use super::v3::{V3PoolInfo, load_pools, price_from_sqrt_price, format_liquidity};

#[component]
pub fn PoolV3Pairs() -> Element {
    let mut show_zero_liq = use_signal(|| false);
    let pools = use_v3_pools();

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
              if (pools.is_loading)() {
                  div { class: "text-gray-300", "Loading..." }
              }

              if let Some(err) = pools.error.read().as_ref() {
                  div { class: "text-red-500 mb-4", "{err}" }
              }

              // ---------- PAIRS LIST ----------
              div { class: "flex flex-col gap-3",
                    {pools.pairs.iter().filter_map(|pair| {
                        let state = pools.pool_state.read().get(&pair.pair_address).cloned().unwrap_or_default();
                        let liquidity = state.liquidity;
                        if  liquidity > 0_u128 || show_zero_liq(){
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
