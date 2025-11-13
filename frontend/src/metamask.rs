use wasm_bindgen::prelude::*;
use serde::Deserialize;
use serde_wasm_bindgen::from_value;

#[derive(Deserialize, Clone, Debug)]
pub struct TokenWrapperInfo {
    pub wrapper: String,
    pub dTokenSymbol: String,
    pub dTokenAddress: String,
    pub dTokenDecimals: u64,
    pub cAssetSymbol: String,
    pub cAssetAddress: String,
    pub cAssetDecimals: u64,
    pub fees: Fees,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Fees {
    pub inBps: u64,
    pub outBps: u64,
}

// Bind JS functions in metamask.js
#[wasm_bindgen(module = "/src/metamask.js")]
extern "C" {
    pub async fn connect_metamask() -> JsValue;
    pub async fn get_token_balance(user: &str, token: &str) -> JsValue;
    pub async fn get_all_wrappers(factory_address: &str) -> JsValue;
    pub async fn wrap_tokens(contract: &str, dToken: &str, amount: &str, cAsset: &str,) -> JsValue;
    pub async fn unwrap_tokens(contract: &str, cAsset: &str, amount: &str, dToken: &str) -> JsValue;
}

pub async fn fetch_wrappers(factory_address: &str) -> Result<Vec<TokenWrapperInfo>, JsValue> {
    let js_val = get_all_wrappers(factory_address).await;
    let parsed: Vec<TokenWrapperInfo> = from_value(js_val)?; // <-- use serde_wasm_bindgen
    Ok(parsed)
}
