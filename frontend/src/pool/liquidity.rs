use alloy::primitives::utils::{format_units, parse_units};
use dioxus::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::components::switch::{Switch, SwitchThumb};
use crate::wallet_context::use_wallet;
use crate::token::{TokenInfo, TokenSelector, TokenSelectorAmount, TokenType};
use crate::metamask::uniswap_v2::{V2PairInfo as PoolInfo};
use crate::metamask::MetamaskInfo;
use crate::metamask::get_token_balance;
use crate::vanillaswap::v2::{unique_pool_tokens, get_ratio, calc_pool_share};
use super::get_pool;

#[derive(Clone, Copy, PartialEq)]
enum LiquidityTab {
    Add,
    Remove,
}

#[component]
pub fn LiquidityPopup(
    mouse_y: Signal<f64>,
    show: Signal<bool>,
    show_zero_liq : Signal<bool>,
    show_balanced : Signal<bool>,
    all_tokens : Signal<bool>,
    on_close: EventHandler<()>,
    info : Signal<MetamaskInfo>,
    pool_list: Signal<Vec<PoolInfo>>,
    pool_info: Signal<Option<PoolInfo>>,
    on_confirm_add_liquidity: EventHandler<(TokenInfo, TokenInfo, String, String)>,
    on_confirm_rem_liquidity: EventHandler<(TokenInfo, String)>,
) -> Element {
    let mut active_tab = use_signal(|| LiquidityTab::Add);
    // when mounted -> visible classes, else hidden classes
    let overlay_cls = if show() {"fixed inset-0 bg-black/60 flex items-center justify-center z-50 transition-opacity duration-200 opacity-100"}else{"hidden"};

    let panel_cls = "bg-gray-900/95 border border-gray-700 rounded-2xl p-6 w-full max-w-md shadow-2xl transform transition-all duration-200 opacity-100 translate-y-0 scale-100 absolute left-1/2 -translate-x-1/2";

    rsx! {
        div { class: "{overlay_cls}",
              div { class: "{panel_cls}",
                    style: format!("top: {}px;", mouse_y()),
                    button { class: "absolute top-4 right-4 text-white transition", onclick: move |_| on_close(()), "✕" }
                    div { class: "mt-4 flex rounded-xl bg-gray-800/60 p-1",
                          button {
                              class: format!(
                                  "flex-1 py-2 rounded-lg text-sm font-medium transition {}",
                                  if active_tab() == LiquidityTab::Add {
                                      "bg-gray-900 text-white"
                                  } else {
                                      "text-gray-400 hover:text-white"
                                  }
                              ),
                              onclick: move |_| active_tab.set(LiquidityTab::Add),
                              "Add Liquidity"
                          }

                          button {
                              class: format!(
                                  "flex-1 py-2 rounded-lg text-sm font-medium transition {}",
                                  if active_tab() == LiquidityTab::Remove {
                                      "bg-gray-900 text-white"
                                  } else {
                                      "text-gray-400 hover:text-white"
                                  }
                              ),
                              onclick: move |_| active_tab.set(LiquidityTab::Remove),
                              "Remove Liquidity"
                          }
                    }
                    match active_tab() {
                        LiquidityTab::Add => rsx! {
                            AddLiquidityView{
                                show_zero_liq,
                                show_balanced,
                                all_tokens,
                                info,
                                pool_list,
                                pool_info,
                                on_confirm: on_confirm_add_liquidity,
                            }
                        },
                        LiquidityTab::Remove => rsx! {

                        }
                    }
              }
        }
    }
}

pub fn RemoveLiquidityView(
    info : Signal<MetamaskInfo>,
    pool_list: Signal<Vec<PoolInfo>>,
    pool_info: Signal<Option<PoolInfo>>,
    on_confirm: EventHandler<(TokenInfo, String)>,
) -> Element {

    rsx! {
    }

}

#[component]
pub fn AddLiquidityView(
    show_zero_liq : Signal<bool>,
    show_balanced : Signal<bool>,
    all_tokens : Signal<bool>,
    info : Signal<MetamaskInfo>,
    pool_list: Signal<Vec<PoolInfo>>,
    pool_info: Signal<Option<PoolInfo>>,
    on_confirm: EventHandler<(TokenInfo, TokenInfo, String, String)>,
) -> Element {
    let mut token_a = use_signal(|| None as Option<TokenInfo>);
    let mut token_b = use_signal(|| None as Option<TokenInfo>);
    let mut token_a_reserve = use_signal(|| "0".to_string());
    let mut amount_a = use_signal(|| "0".to_string());
    let mut amount_b = use_signal(|| "0".to_string());
    // react on pool_info change
    use_effect(move || {
        let pool_info = pool_info();
        spawn_local(async move {
            if let Some(pool) = pool_info{
                if let Some(from_sel) = token_a() &&
                    let Some(to_sel) = token_b() &&
                    let Some(token_pool) = get_pool(&from_sel.address, &to_sel.address, &pool_list()) &&
                    token_pool.pair_address == pool.pair_address
                {

                }else{
                    token_a.set(
                        Some(TokenInfo {
                            symbol: pool.symbol0.clone().unwrap_or("???".into()),
                            address: pool.token0.clone(),
                            decimals: pool.decimals0.unwrap_or(18),
                            token_type: TokenType::CAsset,

                        })
                    );
                    token_b.set(
                        Some(TokenInfo {
                            symbol: pool.symbol1.clone().unwrap_or("???".into()),
                            address: pool.token1.clone(),
                            decimals: pool.decimals1.unwrap_or(18),
                            token_type: TokenType::CAsset,

                        })
                    );
                }
                if let Some(token_a) = token_a() && token_a.address == pool.token0{
                    token_a_reserve.set(pool.reserve0.clone().unwrap_or("0".to_string()));
                }else if let Some(token_a) = token_a() && token_a.address == pool.token1{
                    token_a_reserve.set(pool.reserve1.clone().unwrap_or("0".to_string()));
                }else{
                    token_a_reserve.set("0".to_string());
                }
            }
        });
    });

    use_effect(move || {
        let token_a = token_a();
        let token_b = token_b();
        spawn_local(async move {
            if let Some(from_sel) = token_a &&
                let Some(to_sel) = token_b
            {
                let new_pool = get_pool(&from_sel.address, &to_sel.address, &pool_list());
                if new_pool.is_none() || pool_info().is_none() || new_pool.clone().unwrap() != pool_info().unwrap(){
                    pool_info.set(new_pool);
                }

            }
        });
    });

    // Auto-ratio sync if pool exists
    let mut sync_ratio = move |changed_a: bool| {
        if let Some(pool) = pool_info() {
            if let Some(reserve0) = pool.reserve0 &&
                let Some(reserve1) = pool.reserve1 &&
                let Some(token_a) =  token_a() &&
                let Some(token_b) =  token_b() &&
                let Ok(reserve0) = parse_units(&reserve0, 0) &&
                let Ok(reserve1) = parse_units(&reserve1, 0) &&
                let Ok(amount0) = parse_units(&amount_a(), token_a.decimals as u8) &&
                let Ok(amount1) = parse_units(&amount_b(), token_b.decimals as u8)
            {
                if changed_a {
                    if reserve0.get_absolute() > 0 {
                        amount_b.set(format_units(amount0.get_absolute() * reserve1.get_absolute() / reserve0.get_absolute(), token_b.decimals as u8).unwrap_or("0".to_string()));
                    }
                } else {
                    if reserve1.get_absolute() > 0 {
                        amount_a.set(format_units(amount1.get_absolute() * reserve0.get_absolute() / reserve1.get_absolute(), token_a.decimals as u8).unwrap_or("0".to_string()));
                    }
                }
            }
        }
    };
    let from_options = unique_pool_tokens(&None, &pool_list.read(), &show_zero_liq.read(), &true, &false, info().chain_id);
    let to_options = if all_tokens(){
        from_options.clone()
    }else{
        unique_pool_tokens(&token_a.read(), &pool_list.read(), &show_zero_liq.read(), &true, &false, info().chain_id)
    };
    let (reserve0, reserve1) = if let Some(pool) = pool_info(){
        if let Some(token_a) = token_a() && token_a.address == pool.token0{
            (pool.reserve0.unwrap_or_default(),pool.reserve1.unwrap_or_default())
        }else{
            (pool.reserve1.unwrap_or_default(),pool.reserve0.unwrap_or_default())
        }
    }else{
        (amount_a().clone(),amount_b().clone())
    };

    rsx! {
        // TOKEN A SELECTOR
        div { class: "space-y-1 mt-3",
              label { class: "text-sm text-gray-400", "Token A" }
              TokenSelectorAmount {
                  info,
                  token_list: from_options,
                  selected: token_a,
                  amount : amount_a,
                  on_select_token: move || {
                      //token_a.set(Some(t));
                      if let Some(from_sel) = token_a() &&
                          let Some(to_sel) = token_b() &&
                          let Some(pool) = get_pool(&from_sel.address, &to_sel.address, &pool_list())
                      {
                          pool_info.set(Some(pool));
                      }else{
                          token_b.set(None);
                      }
                      sync_ratio(false);
                  },
                  on_select_amount: move || {sync_ratio(true);},
                  allow_manual: true
              }
        }
        // TOKEN B SELECTOR
        div { class: "space-y-1 mt-2",
              div { class: "flex items-center justify-between",
                    label { class: "text-sm text-gray-400", "Token B" }
                    div { class: "flex items-center gap-2",
                          span { class: "text-gray-200 text-sm", "Show all tokens" }
                          Switch {
                              checked: all_tokens(),
                              on_checked_change: move |new_state| all_tokens.set(new_state),
                              SwitchThumb {}
                          }
                    }
              }
              TokenSelectorAmount {
                  info,
                  token_list: to_options,
                  selected: token_b,
                  amount : amount_b,
                  on_select_token: move || {sync_ratio(true);},
                  on_select_amount: move || {sync_ratio(false);},
                  allow_manual: true
              }
        }

        // POOL INFO
        if let Some(token_a) = token_a() && let Some(token_b) = token_b()
        {
            div { class: "text-sm text-gray-400 bg-gray-800/40 p-3 rounded-xl border border-gray-700 mt-3 space-y-1",
                  p { "{token_a.symbol.clone()} per {token_b.symbol.clone()} : {get_ratio(reserve1.clone(),reserve0.clone()) }" }
                  p { "{token_b.symbol.clone()} per {token_a.symbol.clone()} : {get_ratio(reserve0.clone(),reserve1.clone()) }" }
                  p { "Share : {calc_pool_share(amount_a().clone(), token_a_reserve().clone(), token_a.decimals as u8) }%" }
            }

        }

        // SUBMIT
        button {
            class: "w-full bg-purple-600 hover:bg-purple-500 py-3 rounded-xl text-white font-semibold mt-4 disabled:opacity-40",
            disabled: token_a().is_none() || token_b().is_none() || amount_a().is_empty() || amount_b().is_empty(),
            onclick: move |_| {
                if let (Some(a), Some(b)) = (token_a(), token_b()) {
                    on_confirm((a, b, amount_a(), amount_b()));
                }
            },
            if pool_info().is_none(){
                "Create & Add Liquidity"
            }else{
                "Add Liquidity"
            }
        }
    }
}
