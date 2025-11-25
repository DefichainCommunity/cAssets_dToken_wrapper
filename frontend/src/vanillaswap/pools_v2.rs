use dioxus::prelude::*;
use wasm_bindgen_futures::spawn_local;
use alloy::primitives::{utils::parse_units,U256};
use crate::components::switch::{Switch, SwitchThumb};
use crate::metamask::uniswap_v2::{V2PairInfo, get_uniswap_v2_pairs};
use crate::wallet_context::use_wallet;
use super::v2::{use_v2_pools, is_zero_or_empty};

#[component]
pub fn PoolV2Pairs() -> Element {
    // let mut is_loading = use_signal(|| false);
    // let mut pairs = use_signal(|| Vec::<PairInfo>::new());
    // let mut error = use_signal(|| None::<String>);
    let mut show_zero_liq = use_signal(|| false);
    // let mut router_address = use_signal(|| "".to_string());
    let pools = use_v2_pools();
    // use_effect(move || {
    //     let wallet = use_wallet();
    //     let _info = (wallet.info)().clone();
    //     spawn_local(async move {
    //         let info =( wallet.info)().clone();
    //         log::debug!("Chain ID:{}", info.chain_id);
    //         pairs.set(vec![]);
    //         if info.chain_id == 1130{ // MainNet
    //             router_address.set("0x3E8C92491fc73390166BA00725B8F5BD734B8fba".to_string());
    //         }else if  info.chain_id == 1131{ // TestNet
    //             router_address.set("0x79208eADd9FbC29116108433a38Af62D0fD83850".to_string());
    //         }else{
    //             router_address.set("".to_string());
    //         }

    //         if !info.address.is_empty() && !router_address.is_empty() {
    //             is_loading.set(true);
    //             log::debug!("Router address {}", router_address);
    //             match get_uniswap_v2_pairs(&router_address()).await {
    //                 Ok(list) => pairs.set(list),
    //                 Err(e) => error.set(Some(e.to_string())),
    //             }
    //             is_loading.set(false);

    //         }
    //     })
    // });

    rsx! {
        div { class: "p-8 mt-12 glass w-full max-w-4xl flex flex-col gap-6 items-stretch flex-col-sm",
              h2 { class: "text-3xl font-bold text-center mb-6", "V2 PoolPairs" }
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
                    for pair in pools.pairs.read().iter().filter(|p| {
                        show_zero_liq() || (!is_zero_or_empty(&p.reserve0) && !is_zero_or_empty(&p.reserve1))
                        // if show_zero_liq() {
                        //     true
                        // } else {
                        //     !is_zero_or_empty(&p.reserve0) &&
                        //     !is_zero_or_empty(&p.reserve1)
                        // }
                    }) {
                        div { class: "p-4 bg-gray-900/60 border border-gray-800 rounded-xl shadow-md
                                    flex flex-col gap-2 hover:bg-gray-900 transition-colors duration-200",
                              // row 1
                              div { class: "flex items-center",

                                    // TOKEN 0
                                    div { class: "flex items-center gap-2",
                                          div { class: "text-gray-200 font-semibold text-lg",
                                                "{ pair.symbol0.clone().unwrap_or(\"?\".into()) }"
                                          }
                                    }

                                    // Slash between tokens
                                    div { class: "text-gray-500 font-bold text-xl", "/" }

                                    // TOKEN 1
                                    div { class: "flex items-center gap-2",
                                          div { class: "text-gray-200 font-semibold text-lg",
                                                "{ pair.symbol1.clone().unwrap_or(\"?\".into()) }"
                                          }
                                    }

                                    // ADDRESS RIGHT SIDE
                                    div { class: "ml-auto text-xs text-gray-500 tracking-wide",
                                          "{pair.pair_address}"
                                    }
                              }
                              // ROW 2
                              div { class: "flex items-center text-xs font-bold text-gray-500",
                                    div { class: "w-1/3",
                                          "Reserve0: { pair.reserve0.clone().unwrap_or(\"0\".into())  }"
                                    }
                                    div { class: "w-1/3",
                                          "Reserve1: { pair.reserve1.clone().unwrap_or(\"0\".into())  }"
                                    }

                              }
                              // // ROW 3
                              // div { class: "flex items-center text-xs font-bold text-gray-500",



                              //       // COLUMN 3 — PRICE
                              //       div { class: "w-1/2 text-right",
                              //             "Price token0/token1: "
                              //       }
                              // }

                        }
                    }
              }
        }
    }
}
