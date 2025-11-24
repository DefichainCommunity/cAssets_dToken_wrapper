use dioxus::prelude::*;
use std::collections::BTreeMap;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use serde_wasm_bindgen::from_value;
use wasm_bindgen::closure::Closure;
use crate::metamask::uniswap_v3::V3PoolState;
use crate::metamask::{connect_metamask, MetamaskInfo, js_on_chain_changed, js_on_accounts_changed};
use crate::vanillaswap::swap_v2::PoolV2Swap;
use crate::vanillaswap::swap_v3::PoolV3Swap;
use crate::vanillaswap::v3::{use_sync_v3_pools, V3PoolInfo, UniswapV3PoolContext};
use crate::vanillaswap::v2::{use_sync_v2_pools, UniswapV2PoolContext};
use crate::metamask::uniswap_v2::V2PairInfo;
use crate::wrapper::Wrapper;
use crate::vanillaswap::pools_v2::PoolV2Pairs;
use crate::vanillaswap::pools_v3::PoolV3Pairs;
use crate::wallet_context::{WalletContext, use_wallet};


#[derive(Clone, PartialEq)]
enum Tab {
    Wrap,
    SwapV2,
    PoolV2,
    SwapV3,
    PoolV3,
}

fn button_class(tab: Tab, active: Tab) -> String {
    if tab == active {
        "px-4 py-2 rounded-xl bg-purple-600 text-white".into()
    } else {
        "px-4 py-2 rounded-xl bg-gray-700 text-gray-300 hover:bg-gray-600".into()
    }
}

pub fn init_metamask_listeners() {
    let mut wallet = use_wallet();

    // Chain changed
    let chain_closure = Closure::wrap(Box::new(move |chain_id: u32| {
        let mut info_data = (wallet.info)();
        log::info!("Network changed to {}", chain_id);
        info_data.chain_id = chain_id;
        wallet.info.set(info_data);
    }) as Box<dyn FnMut(u32)>);
    js_on_chain_changed(&chain_closure);
    chain_closure.forget(); // keep alive

    // Accounts changed
    let accounts_closure = Closure::wrap(Box::new(move |accounts: Vec<JsValue>| {
        let mut info_data = (wallet.info)();
        let addrs: Vec<String> = accounts.iter()
            .filter_map(|a| a.as_string())
            .collect();
        log::info!("Accounts changed: {:?}", addrs);
        info_data.address = addrs.first().unwrap_or(&"".to_string()).to_string();
        wallet.info.set(info_data);
    }) as Box<dyn FnMut(Vec<JsValue>)>);
    js_on_accounts_changed(&accounts_closure);
    accounts_closure.forget(); // keep alive
}

#[component]
pub fn App() -> Element {

    let mut active_tab = use_signal(|| Tab::Wrap);

    let info = use_signal(|| MetamaskInfo::default());
    let short = info.with(|info| {
        let addr = info.address.clone();
        if addr.len() >= 10 {
            format!("{}...{}", &addr[0..6], &addr[addr.len() - 4..])
        } else {
            addr.clone()
        }
    });
    let is_connecting = use_signal(|| false);
    use_context_provider(|| WalletContext {
        info: info.clone(),
        is_connecting,
    });

    init_metamask_listeners();

    // uniswap v3
    {
        let pairs       = use_signal(|| Vec::<V3PoolInfo>::new());
        let pool_state  = use_signal(|| BTreeMap::new());
        let is_loading  = use_signal(|| false);
        let error       = use_signal(|| None);
        let router_address = use_signal(|| "".to_string());

        use_context_provider(|| UniswapV3PoolContext {
            pairs,
            pool_state,
            router_address,
            is_loading,
            error,
        });

        use_sync_v3_pools();
    }

    {
        let is_loading = use_signal(|| false);
        let pairs = use_signal(|| Vec::<V2PairInfo>::new());
        let error = use_signal(|| None::<String>);
        let router_address = use_signal(|| "".to_string());
        use_context_provider(|| UniswapV2PoolContext {
            pairs,
            router_address,
            is_loading,
            error,
        });

        use_sync_v2_pools();
    }


    let on_connect = move |_| {
        let mut wallet = use_wallet();
        spawn_local(
            async move {
                //let _res = connect_metamask().await;
                match connect_metamask().await{
                    Ok(meta_info) => wallet.info.set(meta_info),
                    Err(err) => log::error!("MetaMask connect failed: {:?}", err),
                }
            }
        )
    };

    rsx! {
        document::Stylesheet { href: asset!("/assets/tailwind.css") }
        document::Stylesheet { href: asset!("/assets/dx-components-theme.css") }
        div {
            class: "relative min-h-screen flex flex-col bg-gradient-to-br from-bg-from via-bg-mid to-bg-to text-white",

            // ---- FIXED TOP BAR ----
            div {
                class: "fixed top-0 left-0 w-full flex items-center justify-between
                    px-8 py-4 backdrop-blur-sm bg-black/20 z-50",

                div { class: "relative",
                      h1 {
                          class: "text-xl font-bold tracking-wide text-defichain",
                          "DeFiChainCommunityServices"
                      }
                      a {
                          href: "https://www.dex-trading.live/",
                          target: "_blank",
                          rel: "noopener noreferrer",
                          class: "absolute left-1/2 -translate-x-1/2 top-full -mt-2
                            text-ms text-dextradinglive hover:text-dextradinglive-hover
                            transition whitespace-nowrap",
                          "Powered by Dex Trading Live"
                      }
                }

                // Tabs
                div { class: "flex space-x-4 mt-6",
                      button {
                          class: button_class(Tab::Wrap, active_tab()),
                          onclick: move |_| active_tab.set(Tab::Wrap),
                          "Wrap"
                      }
                      button {
                          class: button_class(Tab::SwapV2, active_tab()),
                          onclick: move |_| active_tab.set(Tab::SwapV2),
                          "SwapV2"
                      }

                      button {
                          class: button_class(Tab::PoolV2, active_tab()),
                          onclick: move |_| active_tab.set(Tab::PoolV2),
                          "PoolsV2"
                      }

                      button {
                          class: button_class(Tab::SwapV3, active_tab()),
                          onclick: move |_| active_tab.set(Tab::SwapV3),
                          "SwapV3"
                      }

                      button {
                          class: button_class(Tab::PoolV3, active_tab()),
                          onclick: move |_| active_tab.set(Tab::PoolV3),
                          "PoolsV3"
                      }


                }

                if !short.is_empty() {
                    button { class: "btn-gradient",
                             "{short}"
                             div {class: "text-xs  ",
                                 "ChainID:{info().chain_id}"
                             }
                    }


                } else {
                    button { class: "btn-gradient", onclick: on_connect, "Connect Wallet" }
                }
            }

            // ---- SCROLLABLE CONTENT ----
            div {
                class: "flex-1 w-full overflow-y-auto
                    flex flex-col items-center
                    pt-[var(--header-height)] px-4 pb-10 fade-slide-in",

                match *active_tab.read() {
                    Tab::Wrap => rsx!(Wrapper{}),
                    Tab::SwapV2 => rsx!(PoolV2Swap{}),
                    Tab::PoolV2 => rsx!(PoolV2Pairs{}),
                    Tab::SwapV3 => rsx!(PoolV3Swap{}),
                    Tab::PoolV3 => rsx!(PoolV3Pairs{}),
                }
            }
        }
    }
}
