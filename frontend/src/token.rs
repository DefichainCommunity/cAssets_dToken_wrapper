use dioxus::prelude::*;
use wasm_bindgen_futures::spawn_local;
use serde::{Serialize, Deserialize};
use crate::metamask::{MetamaskInfo, get_token_balance};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TokenInfo {
    pub symbol: String,
    pub address: String,
    pub decimals: u64,
    pub token_type : TokenType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TokenType{
    Native,
    DToken,
    CAsset,
}

#[component]
pub fn TokenSelectorAmount(
    info : Signal<MetamaskInfo>,
    token_list: Vec<TokenInfo>,
    selected: Signal<Option<TokenInfo>>,
    amount: Signal<String>,
    on_select_token: EventHandler<()>,
    on_select_amount: EventHandler<()>,
    allow_manual: bool,
) -> Element {
    let mut balance = use_signal(|| "0".to_string());

    use_effect(move || {
        let selected = selected();
        spawn_local(async move {
            if let Some(sel) = selected
                && let Ok(bal) = get_token_balance(&info().address, &sel.address, matches!(sel.token_type, TokenType::Native)).await {
                    log::debug!("GetTokenBalance of address {} for token address {} :{:?}",info().address, sel.address, bal);
                    balance.set(bal);
                }
        });
    });

    rsx! {
        // TOKEN SELECTOR
        TokenSelector { token_list, selected, on_select: on_select_token, allow_manual}
        // Amounts
        if let Some(_t) = selected() {
            div { class: "space-y-1 mt-3",
                  div { class: "flex justify-between text-xs text-gray-400",
                        span { "Amount" }
                        button { class: "px-3 py-1 bg-white/10 rounded-lg text-white", onclick: move |_| {
                            amount.set(balance());
                            on_select_amount(());
                        }, "Max" }
                        span { "Balance: {balance}" }
                  }
                  input {
                      class: "w-full bg-gray-800 border border-gray-700 rounded-xl p-3 text-white",
                      value: "{amount()}",
                      oninput:  move |ev| {
                          amount.set(ev.value());
                          on_select_amount(());
                      },
                      placeholder: "0.0",
                  }
            }
        }
    }
}

#[component]
pub fn TokenSelector(
    token_list: Vec<TokenInfo>,
    selected: Signal<Option<TokenInfo>>,
    on_select: EventHandler<()>,
    allow_manual: bool,
) -> Element {
    let mut query = use_signal(|| String::new());
    let mut last_query = use_signal(|| String::new());
    let mut open = use_signal(|| false);

    let filtered_tokens: Vec<TokenInfo> = token_list.iter()
        .filter(|t| {
            let q = query().to_lowercase();
            if q.starts_with("0x"){
                t.address.to_lowercase().contains(&q)
            }else{
                t.symbol.to_lowercase().contains(&q)
            }
        })
        .cloned().collect();

    use_effect(move || {
        let selected = selected();
        spawn_local(async move {
            log::debug!("selected changed old query {:?} - selected {:?}", query, selected);
            if let Some(selected) = selected{
                query.set(selected.symbol.clone());
            }else{
                query.set(String::new());
            }
        });
    });

    let on_select_internal = move ||{ //token: TokenInfo|{
        open.set(false);
        if let Some(t) = selected(){
            query.set(t.symbol.clone());
        }
        on_select(());
    };

    rsx! {
        div { class: "relative",
              // clicking anywhere outside closes dropdown
              onclick: move |_| open.set(false),

              // INNER WRAPPER — stops propagation so clicks inside don't close it
              div { class: "relative w-full", //onclick: move |ev| ev.stop_propagation(),
                    // INPUT
                    input {
                        class: "w-full bg-gray-800 border border-gray-700 rounded-xl p-3 text-white placeholder:text-gray-500 focus:border-purple-500 outline-none transition",
                        placeholder: "Search or paste address...",
                        value: "{query()}",
                        // onclick:  move |_| query.set("".to_string()),
                        onclick: move |ev| {
                            ev.stop_propagation();
                            if (open()){
                                if query().is_empty(){
                                    query.set(last_query());
                                }
                            }else{
                                last_query.set(query());
                                query.set("".to_string());
                            }
                            open.set(!open());
                        },
                        oninput: move |ev| {
                            query.set(ev.value());
                            open.set(true);
                        },
                    }

                    if open() {
                        div {
                            class: "absolute left-0 right-0 mt-1 z-50 rounded-xl border border-gray-700 bg-gray-900 shadow-xl",

                            // prevent outside click from closing dropdown while interacting inside
                            onclick: move |ev| ev.stop_propagation(),

                            /* FIXED HEADER */
                            div {
                                class: "px-4 py-2 text-xs uppercase tracking-wide text-gray-500 bg-gray-800/50 border-b border-gray-700",
                                "Tokens"
                            }

                            /* SCROLL AREA */
                            div {
                                class: "max-h-72 overflow-y-auto divide-y divide-gray-800",
                                onclick: move |ev| ev.stop_propagation(),
                                /* manual address option */
                                if allow_manual && !query().is_empty() {
                                    button {
                                        class: "w-full px-4 py-3 text-left text-sm text-gray-300 hover:bg-gray-800",
                                        onclick: move |_| {
                                            let address = query();
                                            let t = TokenInfo{
                                                symbol: "unknown".to_string(),
                                                address: address,
                                                decimals: 18,
                                                token_type : TokenType::CAsset,
                                            };
                                            selected.set(Some(t.clone()));
                                            on_select(());
                                            open.set(false);
                                        },

                                        "Use address: {query()}"
                                    }
                                }

                                /* TOKEN LIST ITEMS */
                                { filtered_tokens.iter().map(|token| {rsx! {TokenListItem {token: token.clone(), on_select: on_select_internal.clone(), selected: selected.clone()}}})}
                            }
                        }
                    }
              }
        }
    }
}

#[component]
pub fn TokenListItem(
    token: TokenInfo,
    on_select: EventHandler<()>,
    selected: Signal<Option<TokenInfo>>,
) -> Element {

    rsx! {
        button { class: "w-full px-4 py-3 hover:bg-gray-800 transition flex items-center gap-3",
                 onclick: move |_| {
                     log::debug!("New token selected :{:?}", token);
                     selected.set(Some(token.clone()));
                     on_select(());//token.clone());
                 },


                 // Placeholder icon
                 div { class: "w-6 h-6 rounded-full bg-gray-700", "?"}


                 div { class: "flex flex-col text-left",
                       span { class: "text-white text-sm", "{token.symbol}" }
                       // span { class: "text-gray-400 text-xs", "{symbol}" }
                 }
        }
    }
}
