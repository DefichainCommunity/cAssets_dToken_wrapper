use dioxus::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::metamask::{connect_metamask, wrap_tokens, unwrap_tokens};

#[component]
pub fn App() -> Element {
    // Signals for input
    let address = use_signal(|| "".to_string());
    let mut underlying = use_signal(|| "".to_string());
    let mut wrapped = use_signal(|| "".to_string());
    let mut amount = use_signal(|| "".to_string());

    // Signal for transaction status
    let tx_status = use_signal(|| "".to_string());

    let contract_address = "0xYourContractAddressHere";

    rsx! {
        div {
            h2 { "dToken Wrapper UI" }

            // Connect MetaMask
            button {
                onclick: move |_| {
                    let mut address = address.clone();
                    spawn_local(async move {
                        let addr = connect_metamask().await;
                        address.set(addr.as_string().unwrap_or_default());
                    });
                },
                "Connect MetaMask"
            }
            p { "Connected: {address}" }

            // Input for underlying token
            input {
                placeholder: "Underlying token address",
                value: "{underlying}",
                oninput: move |e| underlying.set(e.value())
            }

            // Input for amount
            input {
                placeholder: "Amount",
                value: "{amount}",
                oninput: move |e| amount.set(e.value())
            }

            // Wrap button
            button {
                onclick: move |_| {
                    let underlying = underlying.clone();
                    let amount = amount.clone();
                    let mut tx_status = tx_status.clone();
                    spawn_local(async move {
                        tx_status.set("Wrapping...".to_string());
                        let res = wrap_tokens(contract_address, &underlying.read(), &amount.read()).await;
                        tx_status.set(format!("Wrap done: {:?}", res));
                    });
                },
                "Wrap Tokens"
            }

            // Input for wrapped token
            input {
                placeholder: "Wrapped token address",
                value: "{wrapped}",
                oninput: move |e| wrapped.set(e.value())
            }

            // Unwrap button
            button {
                onclick: move |_| {
                    let wrapped = wrapped.clone();
                    let amount = amount.clone();
                    let mut tx_status = tx_status.clone();
                    spawn_local(async move {
                        tx_status.set("Unwrapping...".to_string());
                        let res = unwrap_tokens(contract_address, &wrapped.read(), &amount.read()).await;
                        tx_status.set(format!("Unwrap done: {:?}", res));
                    });
                },
                "Unwrap Tokens"
            }

            // Transaction status display
            p { "Transaction Status: {tx_status}" }
        }
    }
}
