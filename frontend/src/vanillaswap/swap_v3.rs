use dioxus::prelude::*;
use wasm_bindgen_futures::spawn_local;
use alloy::primitives::{utils::parse_units,U256};
use crate::components::switch::{Switch, SwitchThumb};
use crate::metamask::{
    get_token_balance,
    uniswap_v3::uniswap_v3_swap_tokens
};
use crate::wrapper::{TokenInfo, TokenType};
use crate::wallet_context::use_wallet;
use super::v3::{approx_amount_out, unique_pool_tokens, use_v3_pools};

#[component]
pub fn PoolV3Swap() -> Element {
    let wallet = use_wallet();
    let pools = use_v3_pools();

    let mut show_zero_liq = use_signal(|| false);
    let tx_status = use_signal(|| "".to_string());

    let mut token_a = use_signal(|| None as Option<TokenInfo>);
    let mut token_b = use_signal(|| None as Option<TokenInfo>);
    let balance = use_signal(|| "0.0".to_string());
    let mut amount_in = use_signal(|| "".to_string()); // human readable
    let mut amount_out = use_signal(|| "0".to_string());
    let mut slippage_percent = use_signal(|| 1.0f64); // default 1.0%
    let mut calculating = use_signal(|| false);

    // Helper to compute estimated amount out using reserves
    use_effect(move || {
        let _amount_in = amount_in();
        let _token_a = token_a();
        let _token_b = token_b();

        spawn_local(async move {
            calculating.set(true);
            let new_amount = approx_amount_out(&amount_in.read(),&token_a.read(), &token_b.read(), &pools.pairs.read(), &pools.pool_state.read());
            log::debug!("New Amount out : {}", new_amount);
            amount_out.set(new_amount);
            calculating.set(false);
        });
    });

    // react on address, from token or balance changes
    use_effect(move || {
        let from_sel = token_a().clone();
        let wallet = use_wallet();
        let mut balance = balance;

        spawn_local(async move {
            if let Some(from_sel) = from_sel
                && let Ok(bal) = get_token_balance(&(wallet.info)().address, &from_sel.address, matches!(from_sel.token_type, TokenType::Native)).await {
                    log::debug!("GetTokenBalance of address {} for token address {} :{:?}",(wallet.info)().address, from_sel.address, bal);
                    balance.set(bal);
                }
        });
    });

    let on_max_click = move |_| {amount_in.set(balance.read().clone());};

    // Swap execution
    let on_swap = {
        let wallet = use_wallet();
        let token_a = token_a.clone();
        let token_b = token_b.clone();
        let amount_in = amount_in.clone();
        let slippage_percent = slippage_percent.clone();
        let is_loading = pools.is_loading.clone();
        move |_| {
            let wallet = wallet.clone();
            let token_a = token_a.clone();
            let token_b = token_b.clone();
            let amount_in = amount_in.clone();
            let slippage_percent = slippage_percent.clone();
            let mut is_loading = is_loading.clone();
            let mut tx_status = tx_status.clone();


            spawn_local(async move {
                let info =  (wallet.info)().clone();
                if info.address.is_empty() {
                    log::error!("wallet not connected");
                    return;
                }
                let a = token_a.read().clone();
                let b = token_b.read().clone();
                if a.is_none() || b.is_none() {
                    log::error!("select tokens");
                    return;
                }

                let a = a.unwrap();
                let b = b.unwrap();

                is_loading.set(true);

                if let Ok(amount_in) = parse_units(&amount_in(), a.decimals as u8) &&
                    let Ok(amount_out) = parse_units(&amount_out(), b.decimals as u8) &&
                    let Some(pool) = pools.pairs.iter().find(|p| (p.token0 == a.address && p.token1 == b.address) || (p.token0 == b.address && p.token1 == a.address))

                {
                    let mul = U256::from(10_000) - U256::from(slippage_percent()*100.0);
                    let amount_out_min = amount_out.get_absolute() * mul / U256::from(10_000);
                    log::debug!("Amount out min: {}", amount_out_min);
                    tx_status.set("Swapping".to_string());
                    match uniswap_v3_swap_tokens(
                        &a.address.clone(),
                        &b.address.clone(),
                        &amount_in.get_absolute().to_string(),
                        &amount_out_min.to_string(),
                        &pool.fee.to_string(),
                        &(pools.router_address)(),
                        matches!(a.token_type, TokenType::Native),
                        matches!(b.token_type, TokenType::Native),
                    ).await {
                        Ok(jsval) => {
                            log::info!("swap ok: {}", jsval);
                            tx_status.set(format!("{}", jsval));

                        }
                        Err(e) => {
                            log::error!("swap error: {}", e);
                            tx_status.set("Failed".to_string());

                        }
                    }
                }
                is_loading.set(false);
            });
        }
    };

    let from_options = unique_pool_tokens(&None, &pools.pairs.read(), &pools.pool_state.read(), &show_zero_liq.read(), (wallet.info)().chain_id);
    let to_options = unique_pool_tokens(&token_a.read(), &pools.pairs.read(), &pools.pool_state.read(), &show_zero_liq.read(), (wallet.info)().chain_id);
    let from_selected = token_a.read().as_ref().map(|t| serde_json::to_string(&t).unwrap()).unwrap_or_default();
    let to_selected = token_b.read().as_ref().map(|t| serde_json::to_string(&t).unwrap()).unwrap_or_default();
    // UI rendering
    rsx! {
        div { class: "p-8 mt-12 glass w-full max-w-4xl flex flex-col gap-6 items-stretch flex-col-sm",
              h2 { class: "text-3xl font-bold text-center mb-6", "V3 PoolSwap" }
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

              // From Panel
              div { class: "panel flex-1",
                    span { class: "text-sm text-gray-200", "From" }
                    div { class: "mt-3 flex items-center justify-between gap-3",
                          select {
                              class: "flex-1 bg-transparent text-white text-xl font-semibold focus:outline-none",
                              value: "{from_selected}",
                              onchange: move |e| {
                                  if let Ok(sel) = serde_json::from_str::<TokenInfo>(&e.value()) {
                                      if let Some(tok) = from_options.iter().find(|t| **t == sel) {
                                          token_a.set(Some(tok.clone()));
                                          // Reset token B, because A changed
                                          token_b.set(None);
                                      }
                                  }
                              },
                              option { value: "", "Select token A" }
                              { from_options.iter().map(|t| rsx!(
                                  option { value: "{serde_json::to_string(&t).unwrap()}", "{t.symbol}" }
                              )) }
                          }
                    }

                    div { class: "mt-2 flex justify-between items-center",
                          span { class: "text-xs text-gray-200", "Balance: {balance.read()}" },
                          button { class: "px-3 py-1 bg-white/10 rounded-lg text-white", onclick: on_max_click, "Max" }
                    }

                    input {
                        class: "mt-4 w-full bg-transparent text-right text-2xl text-white focus:outline-none",
                        placeholder: "Amount",
                        value: "{amount_in.read()}",
                        oninput: move |e| amount_in.set(e.value().to_string())
                    }
              }


              div { class: "flex items-center justify-center",
                    button {
                        class: "mt-6 rounded-full py-3 text-lg font-semibold rounded-xl btn-gradient",
                        onclick: move |_| {
                            let a = token_a().clone();
                            token_a.set(token_b().clone());
                            token_b.set(a);
                        },
                        "⇅"
                    }
              },

              // To Panel
              div { class: "panel flex-1",
                    span { class: "text-sm text-gray-200", "To" }
                    div { class: "mt-3 flex items-center justify-between gap-3",
                          select {
                              class: "flex-1 bg-transparent text-white text-xl font-semibold focus:outline-none",
                              value: "{to_selected}",
                              onchange: move |e| {
                                  if let Ok(sel) = serde_json::from_str::<TokenInfo>(&e.value()) {
                                      if let Some(tok) = to_options.iter().find(|t| **t == sel) {
                                          token_b.set(Some(tok.clone()));
                                      }
                                  }
                              },
                              option { value: "", "Select token B" }
                              { to_options.iter().map(|t| rsx!(
                                  option { value: "{serde_json::to_string(&t).unwrap()}", "{t.symbol}" }
                              )) }
                          }
                    }
                    div { class: "mt-4 text-2xl text-right text-gray-200", "out≈ {amount_out()}" }
                    div { class: "mt-4 text-2xl text-right text-gray-200",
                          input {
                              class: "mt-4 w-full bg-transparent text-right text-2xl text-white focus:outline-none",
                              value: "{slippage_percent.read()}",
                              oninput: move |e| {
                                  if let Ok(v) = e.value().parse::<f64>() {
                                      slippage_percent.set(v);
                                  }
                              }
                          }
                          div { class: "text-sm text-gray-400", "slippage %" }
                    }


                    // div {
                    //     class: "mt-4 text-lg text-right text-gray-200",
                    //     if !fee().is_empty() && let Some(to_selected) = to_selected() && matches!(to_selected.token_type, TokenType::DToken){
                    //         span { class: "opacity-100", "Fee ≈ {fee()}" }
                    //     }else{
                    //         span { class: "opacity-0", "Fee ≈ 0" }
                    //     }
                    // }
              }

              // Swap button
              div { class: "mt-4 flex gap-2 items-center justify-center",

                    button {
                        class: "mt-6 w-sm py-3 text-lg font-semibold rounded-xl btn-gradient",
                        // class: "ml-auto btn-gradient px-5 py-2 rounded",
                        onclick: on_swap,
                        disabled: *pools.is_loading.read(),
                        if *pools.is_loading.read() { "Swapping..." } else { "Swap" }
                    }
              }
        }
        // Footer
        div {
            class: "fixed bottom-0 left-0 w-full items-left justify-between text-sm backdrop-blur-md",
            // Transaction status display
            p { "Transaction Status: {tx_status.read()}" }
        }

    }
}
