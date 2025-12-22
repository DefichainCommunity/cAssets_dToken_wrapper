use std::collections::BTreeMap;

use dioxus::prelude::*;
use wasm_bindgen_futures::spawn_local;
use alloy::primitives::{utils::parse_units};
use crate::components::switch::{Switch, SwitchThumb};
use crate::metamask::get_tokens_balances;
use crate::metamask::uniswap_v2::{V2PairInfo, uniswap_v2_add_liquidity};
use crate::pool::use_filtered_pairs;
use crate::token::{TokenInfo, TokenType};
use crate::wallet_context::use_wallet;
use super::v2::{use_v2_pools, as_decimal, unique_pool_tokens};
use crate::pool::liquidity::LiquidityPopup;
use crate::config::{ConfigEntry, get_config_entry};

#[component]
pub fn PoolV2Pairs() -> Element {
    let pools = use_v2_pools();
    let mut balances = use_signal(|| BTreeMap::<String,String>::new());
    let wallet = use_wallet();
    let mut show_zero_liq = use_signal(|| false);
    let mut show_balanced = use_signal(|| false);
    let mut all_tokens = use_signal(|| false);
    let mut selected_pool = use_signal(|| None as Option<V2PairInfo>);
    let mut mouse_y = use_signal(|| 0.0);
    let mut show_popup = use_signal(|| false);
    let wrapped_native = get_config_entry((wallet.info)().chain_id, &ConfigEntry::WrappedNativeAddress).to_lowercase();

    let on_confirm_add_liquidity = move |(token_a, token_b, amount_a, amount_b): (TokenInfo, TokenInfo, String, String)| {
        spawn_local(async move {
            log::debug!("{:?}-{:?}",token_a,token_b);
            log::debug!("{}-{}",amount_a,amount_b);
            if let Ok(amount_a) = parse_units(&amount_a, token_a.decimals as u8) &&
                let Ok(amount_b) = parse_units(&amount_b, token_b.decimals as u8)
            {

                match uniswap_v2_add_liquidity(
                    &token_a.address.clone(),
                    &token_b.address.clone(),
                    &amount_a.get_absolute().to_string(),
                    &amount_b.get_absolute().to_string(),
                    &(pools.router_address)(),
                    matches!(token_a.token_type, TokenType::Native),
                    matches!(token_b.token_type, TokenType::Native),
                ).await {
                    Ok(_receipt) => log::info!("Liquidity added!"),
                    Err(e) => log::error!("Failed to add liquidity: {:?}", e),
                }
            }
        });
    };

    let on_confirm_rem_liquidity = move |(pool, share_amount): (TokenInfo, String)|{

    };

    use_effect(move || {
        let wallet = use_wallet();
        let pairs = pools.pairs.read().clone();

        spawn_local(async move {
            let unique_tokens = unique_pool_tokens(&None, &pairs, &show_zero_liq.read(), &true, &true, (wallet.info)().chain_id);
            let unique_tokens = unique_tokens.iter().map(|t| t.address.as_str()).collect();
            if !(wallet.info)().address.is_empty(){
                match get_tokens_balances(&(wallet.info)().address, unique_tokens).await{
                    Ok(bal) => {
                        balances.set(bal);
                    },
                    Err(e) => log::error!("Failed getting balances: {:?}", e),
                }
            }
        });
    });

    let pairs_to_show = use_filtered_pairs(wrapped_native, pools.pairs, balances, show_zero_liq, show_balanced);

    rsx! {

        LiquidityPopup {
            mouse_y: mouse_y,
            show: show_popup,
            show_zero_liq : show_zero_liq,
            show_balanced : show_balanced,
            all_tokens : all_tokens,
            on_close : move |_| {show_popup.set(false)},
            info : wallet.info,
            pool_list: pools.pairs,
            pool_info: selected_pool,
            on_confirm_add_liquidity: on_confirm_add_liquidity,
            on_confirm_rem_liquidity: on_confirm_rem_liquidity,
        }

        div { class: "p-8 mt-12 glass w-full max-w-4xl flex flex-col gap-6 items-stretch flex-col-sm",
              h2 { class: "text-3xl font-bold text-center mb-6", "V2 PoolPairs" }
              div { class: "flex items-center gap-2",
                    span { class: "text-gray-200 text-sm", "Show zero liquidity pools" }
                    Switch {
                        checked: show_zero_liq(),
                        on_checked_change: move |new_state| show_zero_liq.set(new_state),
                        SwitchThumb {}
                    }
                    span { class: "text-gray-200 text-sm", "Show balanced tokens only" }
                    Switch {
                        checked: show_balanced(),
                        on_checked_change: move |new_state| show_balanced.set(new_state),
                        SwitchThumb {}
                    }
                    button {
                        class: "ml-auto px-4 py-2 font-semibold rounded-xl btn-gradient",
                        disabled: *pools.is_loading.read(),
                        onclick: move |ev| {
                            let data = &*ev.data;
                            mouse_y.set(data.page_coordinates().y as f64);
                            selected_pool.set(None);
                            show_popup.set(true);
                            all_tokens.set(true);
                        },
                        "New"
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
                    {pairs_to_show.iter().map(|pair| {
                        let pair = pair.clone();
                        rsx!{
                            div {
                                class: "p-4 bg-gray-900/60 border border-gray-800 rounded-xl shadow-md
                                    hover:bg-gray-900 transition-colors duration-200
                                    grid grid-cols-[minmax(0,1fr)_auto] gap-x-4",
                                // LEFT SIDE
                                div {
                                    class: "flex flex-col gap-1 min-w-0",

                                    // --- ROW 1 ---
                                    div {
                                        class: "flex flex-wrap items-center gap-1 min-w-0",

                                        // TOKEN 0
                                        div { class: "text-gray-200 font-semibold text-lg",
                                              "{ pair.symbol0.clone().unwrap_or(\"?\".into()) }"
                                        }

                                        div { class: "text-gray-500 font-bold text-xl", "/" }

                                        // TOKEN 1
                                        div { class: "text-gray-200 font-semibold text-lg",
                                              "{ pair.symbol1.clone().unwrap_or(\"?\".into()) }"
                                        }

                                        // ADDRESS (wraps when needed)
                                        div {
                                            class: "text-xs text-gray-500 break-all flex-grow ",
                                            "{ pair.pair_address }"
                                        }
                                    }

                                    // --- ROW 2 ---
                                    div {
                                        class: "flex flex-wrap items-center gap-1 text-xs font-bold text-gray-500  ",

                                        div { "Reserve0: { as_decimal(&pair.reserve0.clone().unwrap_or(\"0\".into()), pair.decimals0.unwrap_or(0) as u8) }" }
                                        div { "Reserve1: { as_decimal(&pair.reserve1.clone().unwrap_or(\"0\".into()), pair.decimals0.unwrap_or(0) as u8) }" }
                                    }
                                }

                                // RIGHT SIDE — MANAGE BUTTON
                                div {
                                    class: "row-span-2 flex items-center justify-center shrink-0",

                                    button {
                                        class: "px-4 py-2 font-semibold rounded-xl btn-gradient",
                                        disabled: *pools.is_loading.read(),
                                        onclick: move |ev| {
                                            let data = &*ev.data;
                                            mouse_y.set(data.page_coordinates().y as f64);
                                            selected_pool.set(Some(pair.clone()));
                                            show_popup.set(true);
                                        },

                                        "Manage"
                                    }
                                }
                            }
                        }
                    })}
              }
        }
    }
}
