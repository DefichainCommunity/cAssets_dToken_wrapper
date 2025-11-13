use std::io::empty;
use std::str::FromStr;
use alloy::primitives::{utils::{format_units, parse_units, Unit},U256};
use dioxus::prelude::*;
use wasm_bindgen_futures::spawn_local;
use serde_wasm_bindgen::from_value;
use crate::metamask::{TokenWrapperInfo, connect_metamask, get_token_balance, fetch_wrappers, wrap_tokens, unwrap_tokens};

#[derive(Clone, Debug)]
struct TokenInfo {
    symbol: String,
    address: String,
    decimals: u64,
    token_type : TokenType,
}

#[derive(Clone, Debug)]
enum TokenType{
    DToken,
    CAsset,
}

fn update_pair(from: &String, wrappers: &Vec::<TokenWrapperInfo>) -> (Option<TokenInfo>, Option<TokenInfo>){

    if let Some(tok) = wrappers.iter().find(|t| t.dTokenSymbol == *from || t.cAssetSymbol == *from) {
        let d_token = TokenInfo{
            symbol: tok.dTokenSymbol.clone(),
            address: tok.dTokenAddress.clone(),
            decimals: tok.dTokenDecimals,
            token_type: TokenType::DToken,
        };
        let c_asset = TokenInfo{
            symbol: tok.cAssetSymbol.clone(),
            address: tok.cAssetAddress.clone(),
            decimals: tok.cAssetDecimals,
            token_type: TokenType::CAsset,
        };
        if  tok.dTokenSymbol == *from {
            return (Some(d_token),Some(c_asset))
        }else{
            return (Some(c_asset),Some(d_token))
        }
    }
    (None, None)
}

fn token_pair_to_wrapper(token_a: &Option<TokenInfo>, token_b: &Option<TokenInfo>, wrappers: &Vec::<TokenWrapperInfo>) -> Option<TokenWrapperInfo>{
    if let (Some(token_a), Some(token_b)) = (token_a,token_b){
        return wrappers.iter().find(|w|
                                    (w.cAssetAddress == token_a.address && w.dTokenAddress == token_b.address) ||
                                    (w.cAssetAddress == token_b.address && w.dTokenAddress == token_a.address)
        ).cloned();
    }
    None
}

#[component]
pub fn App() -> Element {
    let factory_address = "0x4826533B4897376654Bb4d4AD88B7faFD0C98528";
    let router_address = "0x998abeb3E57409262aE5b751f60747921B33613E";

    let address = use_signal(|| "".to_string());
    let short = address.with(|addr| {
        if addr.len() >= 10 {
            format!("{}...{}", &addr[0..6], &addr[addr.len() - 4..])
        } else {
            addr.clone()
        }
    });

    let mut fee = use_signal(|| "".to_string());
    let mut amount = use_signal(|| "".to_string());
    let mut amount_out = use_signal(|| "".to_string());

    let mut balance = use_signal(|| "0.0".to_string());
    let mut wrappers = use_signal(|| Vec::<TokenWrapperInfo>::new());
    let mut to_selected = use_signal(|| None as Option<TokenInfo>);
    let mut from_selected = use_signal(|| None as Option<TokenInfo>);
    let mut tx_status = use_signal(|| "".to_string());

    let on_connect = move |_| {
        spawn_local({
            let mut address = address.clone();
            let mut wrappers = wrappers.clone();
            let mut from_selected = from_selected.clone();
            let mut to_selected = to_selected.clone();

            async move {
                let addr = connect_metamask().await;
                address.set(addr.as_string().unwrap_or_default());
                let addr = address.read().clone();
                match fetch_wrappers(factory_address).await {
                    Ok(list) => {
                        if let Some(first) = list.first() {
                            let (from,to) = update_pair(&first.dTokenSymbol, &list);
                            to_selected.set(to);
                            from_selected.set(from);
                            if let Ok(bal) = from_value::<String>(get_token_balance(&addr, &first.dTokenAddress).await) {
                                log::debug!("GetTokenBalance of address {} for token address {} :{:?}",addr, first.dTokenAddress, bal);
                                balance.set(bal);
                            }
                        }
                        wrappers.set(list);
                    },
                    Err(e) => log::error!("Error fetching wrappers: {:?}", e)
                }
            }
        });
    };

    // react on address, from token or balance changes
    use_effect(move || {
        let from_sel = from_selected().clone();
        let addr = address.read().clone();
        let mut balance = balance.clone();

        spawn_local(async move {
            if let Some(from_sel) = from_sel {
                if let Ok(bal) = from_value::<String>(get_token_balance(&addr, &from_sel.address).await) {
                    log::debug!("GetTokenBalance of address {} for token address {} :{:?}",addr, from_sel.address, bal);
                    balance.set(bal);
                }
            }
        });
    });

    // react on from token or amount changes
    use_effect(move || {
        let from_sel = from_selected().clone();
        let curr_amount = amount().clone();
        spawn_local(async move {
            if let Some(wrapper) = token_pair_to_wrapper(&from_selected(), &to_selected(), &wrappers())
                && let Some(from_sel) = from_sel//ected()
                && let Some(to_sel) = to_selected()
                && let Ok(curr_amount) = parse_units(&curr_amount, from_sel.decimals as u8){
                    if curr_amount.get_absolute() == 0{
                        fee.set("".to_string())
                    }else{
                        let payed_fee = if matches!(from_sel.token_type, TokenType::DToken){
                            wrapper.fees.inBps
                        }else{
                            wrapper.fees.outBps
                        };
                        let mut payed_fee = curr_amount.get_absolute() * U256::from(payed_fee) / U256::from(1000);
                        if from_sel.decimals > to_sel.decimals{
                            payed_fee = payed_fee / U256::from(10^(from_sel.decimals - to_sel.decimals))
                        }else if from_sel.decimals < to_sel.decimals{
                            payed_fee = payed_fee * U256::from(10^(to_sel.decimals - from_sel.decimals))
                        }
                        let amount_o = curr_amount.get_absolute() - payed_fee;
                        if let Ok(amount_o) = format_units(amount_o, to_sel.decimals as u8){
                            amount_out.set(amount_o);
                        }
                        if let Ok(payed_fee) = format_units(payed_fee, to_sel.decimals as u8){
                            fee.set(payed_fee);
                        }
                    }
                }else{
                    fee.set("".to_string())
                }
        });
    });

    let accent_for = |kind: &Option<TokenInfo>| -> String {
        if let Some(token) = kind {
            match token.token_type {
                TokenType::DToken => "token-badge-defi flex-1 bg-transparent text-white text-xl font-semibold focus:outline-none".to_string(),
                TokenType::CAsset => "token-badge-cf flex-1 bg-transparent text-white text-xl font-semibold focus:outline-none".to_string(),
            }
        } else {
            "bg-white/6".to_string()
        }
    };

    let on_max_click = move |_| {
        amount.set(balance.read().clone());
    };

    let current_wrappers = wrappers.read().clone();

    rsx! {
        document::Stylesheet { href: asset!("/assets/tailwind.css") }

        div { class: "relative min-h-screen flex flex-col items-center justify-center bg-gradient-to-br from-bg-from via-bg-mid to-bg-to text-white",

              // Top Bar
              div { class: "absolute top-0 left-0 w-full flex items-center justify-between px-8 py-4",
                    h1 { class: "text-xl font-bold tracking-wide text-defichain", "DeFiChainCommunityServices" }
                    if !short.is_empty() {
                        button { class: "btn-gradient", "{short}" }
                    } else {
                        button { class: "btn-gradient", onclick: on_connect, "Connect Wallet" }
                    }
              }

              // Main Card
              div { class: "p-8 mt-12 glass w-full max-w-4xl flex flex-col gap-6 items-stretch flex-col-sm",
                    h2 { class: "text-3xl font-bold text-center mb-6", "dToken ⇄ cAsset" }

                    // From Panel
                    div { class: "panel flex-1",
                          span { class: "text-sm text-gray-200", "From" }
                          div { class: "mt-3 flex items-center justify-between gap-3",
                                select {
                                    value: "{from_selected.read().as_ref().map(|t| t.symbol.clone()).unwrap_or_default()}",
                                    class: "{accent_for(&from_selected.read())} flex-1 bg-transparent text-white text-xl font-semibold focus:outline-none",
                                    onchange: move |e| {
                                        let symbol = e.value().to_string();
                                        let (from,to) = update_pair(&symbol, &wrappers());
                                        from_selected.set(from);
                                        to_selected.set(to);
                                    },
                                    { current_wrappers.iter().map(|t| rsx!(
                                        option { value: "{t.dTokenSymbol}", "{t.dTokenSymbol}" },
                                        option { value: "{t.cAssetSymbol}", "{t.cAssetSymbol}" }
                                    )) }
                                }
                          }

                          div { class: "mt-2 flex justify-between items-center",
                                span { class: "text-xs text-gray-200", "Balance: {balance.read()}" },
                                button { class: "px-3 py-1 bg-white/10 rounded-lg text-white", onclick: on_max_click, "Max" }
                          }

                          input {
                              class: "mt-4 w-full bg-transparent text-right text-2xl text-white focus:outline-none",
                              placeholder: "0.0",
                              value: "{amount.read()}",
                              oninput: move |e| amount.set(e.value().to_string())
                          }

                          div {
                              class: "mt-4 text-lg text-right text-gray-200",
                              if !fee().is_empty() && let Some(from_selected) = from_selected() && matches!(from_selected.token_type, TokenType::DToken){
                                span { class: "opacity-100", "Fee ≈ {fee()}" }
                              }else{
                                span { class: "opacity-0", "Fee ≈ 0" }
                              }
                          }

                    }


                    div { class: "flex items-center justify-center",
                          button {
                              class: "mt-6 rounded-full py-3 text-lg font-semibold rounded-xl btn-gradient",
                              onclick: move |_| {
                                  let a = from_selected().clone();
                                  let b = to_selected().clone();
                                  from_selected.set(b);
                                  to_selected.set(a);
                              },
                              "⇅"
                          }
                    },


                    // To Panel
                    div { class: "panel flex-1",
                          span { class: "text-sm text-gray-200", "To" }
                          div { class: "mt-3 flex items-center justify-between gap-3",
                                span { class: "{accent_for(&to_selected.read())}",
                                       "{to_selected.read().as_ref().map(|t| t.symbol.clone()).unwrap_or_default()}" }
                          }
                          div { class: "mt-4 text-2xl text-right text-gray-200", "≈ {amount_out()}" }

                          div {
                              class: "mt-4 text-lg text-right text-gray-200",
                              if !fee().is_empty() && let Some(to_selected) = to_selected() && matches!(to_selected.token_type, TokenType::DToken){
                                  span { class: "opacity-100", "Fee ≈ {fee()}" }
                              }else{
                                  span { class: "opacity-0", "Fee ≈ 0" }
                              }
                          }
                    }

                    // Swap CTA
                    button {
                        class: "mt-6 w-full py-3 text-lg font-semibold rounded-xl btn-gradient",
                        onclick: move |_| {
                            let mut tx_status = tx_status.clone();
                            if let (Some(from_selected), Some(to_selected)) = (from_selected.read().clone(), to_selected.read().clone()){
                                spawn_local({
                                    async move {
                                        tx_status.set(format!("Exchange ..."));
                                        let res = if matches!(from_selected.token_type, TokenType::DToken){
                                            wrap_tokens(router_address,&from_selected.address.to_string(), &amount.read(),&to_selected.address.to_string()).await
                                        } else {
                                            unwrap_tokens(router_address, &from_selected.address.to_string(), &amount.read(), &to_selected.address.to_string()).await
                                        };
                                        tx_status.set(format!("{:?}", serde_wasm_bindgen::from_value::<String>(res)));
                                        if let Ok(bal) = from_value::<String>(get_token_balance(&address(), &from_selected.address.to_string()).await) {
                                            log::debug!("TokenBalance {:?}",bal);
                                            balance.set(bal);
                                        }

                                    }
                                });
                            }
                        },
                        "⇄"
                    }

              }

              div {
                  class: "fixed bottom-0 left-0 w-full items-left justify-between text-sm backdrop-blur-md",
                  // Transaction status display
                  p { "Transaction Status: {tx_status.read()}" }
              }
        }
    }
}
