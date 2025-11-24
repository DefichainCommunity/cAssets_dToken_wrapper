// src/vanillaswap/swap.rs
use dioxus::prelude::*;
use wasm_bindgen_futures::spawn_local;
use alloy::primitives::{utils::{format_units, parse_units},U256};
use crate::metamask::{PairInfo, get_uniswap_v2_pairs, uniswap_v2_swap_tokens, get_token_balance};
use crate::wrapper::{TokenInfo, TokenType};
use crate::wallet_context::use_wallet;
use super::helpers::get_amount_out;

fn unique_pool_tokens(
    selected_a: &Option<TokenInfo>,
    pairs: &Vec<PairInfo>,
) -> Vec<TokenInfo> {
    let address = if let Some(a) = selected_a{
         a.address.clone()
    }else{
        "".to_string()
    };
    let mut seen = std::collections::HashSet::new();
    let mut out = vec![];

    for p in pairs {
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
    out.sort_by(|a, b| a.symbol.cmp(&b.symbol));
    out
}

fn approx_amount_out(
    amount_in: &String,
    token_a: &Option<TokenInfo>,
    token_b: &Option<TokenInfo>,
    pairs: &Vec<PairInfo>,
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

#[component]
pub fn Swap() -> Element {
    let mut router_address = use_signal(|| "".to_string());

    let mut pairs = use_signal(|| Vec::<PairInfo>::new());
    let mut is_loading = use_signal(|| false);
    let mut error = use_signal(|| None as Option<String>);
    let tx_status = use_signal(|| "".to_string());

    let mut token_a = use_signal(|| None as Option<TokenInfo>);
    let mut token_b = use_signal(|| None as Option<TokenInfo>);
    let mut balance = use_signal(|| "0.0".to_string());
    let mut amount_in = use_signal(|| "".to_string()); // human readable
    let mut amount_out = use_signal(|| "0".to_string());
    let mut slippage_percent = use_signal(|| 1.0f64); // default 1.0%
    let mut calculating = use_signal(|| false);

    // load pairs on mount
    use_effect(move || {
        let wallet = use_wallet();
        // this ensures that we react to address changes
        let _info = (wallet.info)().clone();
        spawn_local(async move {
            let info = (wallet.info)().clone();
            log::debug!("Chain ID:{}", info.chain_id);
            token_a.set(None);
            token_b.set(None);
            balance.set("0.0".to_string());
            pairs.set(vec![]);
            if info.chain_id == 1130{ // MainNet
                router_address.set("0x3E8C92491fc73390166BA00725B8F5BD734B8fba".to_string());
            }else if  info.chain_id == 1131{ // TestNet
                router_address.set("0x79208eADd9FbC29116108433a38Af62D0fD83850".to_string());
            }else{
                router_address.set("".to_string());
            }

            if !info.address.is_empty()  && !router_address.is_empty(){
                is_loading.set(true);
                match get_uniswap_v2_pairs(&router_address()).await {
                    Ok(list) => pairs.set(list),
                    Err(e) => error.set(Some(e.to_string())),
                }
                is_loading.set(false);

            }
        });
    });

    // Helper to compute estimated amount out using reserves
    use_effect(move || {
        let _amount_in = amount_in();
        let _token_a = token_a();
        let _token_b = token_b();

        spawn_local(async move {
            calculating.set(true);
            let new_amount = approx_amount_out(&amount_in.read(),&token_a.read(), &token_b.read(), &pairs.read());
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
                && let Ok(bal) = get_token_balance(&(wallet.info)().address, &from_sel.address).await {
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
        let is_loading = is_loading.clone();
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
                    let Ok(amount_out) = parse_units(&amount_out(), b.decimals as u8)
                {
                    let mul = U256::from(10_000) - U256::from(slippage_percent()*100.0);
                    let amount_out_min = amount_out.get_absolute() * mul / U256::from(10_000);
                    log::debug!("Amount out min: {}", amount_out_min);
                    tx_status.set("Swapping".to_string());
                    match uniswap_v2_swap_tokens(
                        &a.address.clone(),
                        &b.address.clone(),
                        &amount_in.get_absolute().to_string(),
                        &amount_out_min.to_string(),
                        &router_address(),
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

    let from_options = unique_pool_tokens(&None, &pairs.read());
    let to_options = unique_pool_tokens(&token_a.read(), &pairs.read());
    let from_selected = token_a.read().as_ref().map(|t| t.address.clone()).unwrap_or_default();
    let to_selected = token_b.read().as_ref().map(|t| t.address.clone()).unwrap_or_default();
    // UI rendering
    rsx! {
        div { class: "p-8 mt-12 glass w-full max-w-4xl flex flex-col gap-6 items-stretch flex-col-sm",
              h2 { class: "text-3xl font-bold text-center mb-6", "V2 PoolSwap" }
              if *is_loading.read() {
                  div { class: "text-gray-300", "Loading..." }
              }

              if let Some(err) = &*error.read() {
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
                                  log::debug!("Value {}",  e.value());
                                  if let Some(tok) = from_options.iter().find(|t| t.address == e.value()) {
                                      log::debug!("Tok found");
                                      token_a.set(Some(tok.clone()));
                                      // Reset token B, because A changed
                                      token_b.set(None);
                                  }
                              },
                              option { value: "", "Select token A" }
                              { from_options.iter().map(|t| rsx!(
                                  option { value: "{t.address}", "{t.symbol}" }
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
                                  let sym = e.value();
                                  if let Some(tok) = to_options.iter().find(|t| t.address == e.value()) {
                                      token_b.set(Some(tok.clone()));
                                  }
                              },
                              option { value: "", "Select token B" }
                              { to_options.iter().map(|t| rsx!(
                                  option { value: "{t.address}", "{t.symbol}" }
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
                        disabled: *is_loading.read(),
                        if *is_loading.read() { "Swapping..." } else { "Swap" }
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
