use std::error::Error;
use std::collections::BTreeMap;
use dioxus::prelude::WritableExt;
use wasm_bindgen::prelude::*;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_wasm_bindgen::from_value;

use crate::wallet_context::use_wallet;

// Bind JS functions in metamask.js
#[wasm_bindgen(module = "/src/metamask.js")]
extern "C" {
    async fn js_connect_metamask() -> JsValue;
    pub fn js_on_chain_changed(callback: &Closure<dyn FnMut(u32)>);
    pub fn js_on_accounts_changed(callback: &Closure<dyn FnMut(Vec<JsValue>)>);
    async fn js_get_token_balance(user: &str, token: &str) -> JsValue;
    // wrapper
    async fn js_get_all_wrappers(factory_address: &str) -> JsValue;
    async fn js_wrap_tokens(contract: &str, dToken: &str, amount: &str, cAsset: &str,) -> JsValue;
    async fn js_unwrap_tokens(contract: &str, cAsset: &str, amount: &str, dToken: &str) -> JsValue;
    // uniswap v2
    async fn js_get_uniswap_v2_pairs(router_address: &str) -> JsValue;
    async fn js_swap_tokens(token_in: &str, token_out: &str, amount_in: &str,
                            amount_out_min: &str, router_address: &str) -> JsValue;
    //uniswap v3
    async fn js_get_uniswap_v3_pool_states(pools: Vec<JsValue>) -> JsValue;
}

pub fn js_parse<T: DeserializeOwned>(js: JsValue) -> Result<T, String> {
    // Parse the wrapper
    let wrapper: JsReturn =
        serde_wasm_bindgen::from_value(js).map_err(|e| format!("{:?}", e))?;

    if !wrapper.ok {
        return Err(wrapper.value); // This is already a string
    }

    // Now parse the inner JSON
    serde_json::from_str(&wrapper.value).map_err(|e| format!("{:?}", e))
}

#[macro_export]
macro_rules! js_try {
    ($expr:expr => $ty:ty) => {{
        let js_val = $expr.await;
        Ok(js_parse::<$ty>(js_val)?)
    }};
}

fn from_str_to_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    s.parse::<u32>().map_err(serde::de::Error::custom)
}


fn from_str_to_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    s.parse::<u64>().map_err(serde::de::Error::custom)
}

fn from_str_to_opt_u64<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    Ok(Some(s.parse::<u64>().map_err(serde::de::Error::custom)?))
}

fn from_str_to_u128<'de, D>(deserializer: D) -> Result<u128, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    s.parse::<u128>().map_err(serde::de::Error::custom)
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TokenWrapperInfo {
    #[allow(dead_code)]
    pub wrapper: String,
    pub d_token_symbol: String,
    pub d_token_address: String,
    #[serde(deserialize_with = "from_str_to_u64")]
    pub d_token_decimals: u64,
    pub c_asset_symbol: String,
    pub c_asset_address: String,
    #[serde(deserialize_with = "from_str_to_u64")]
    pub c_asset_decimals: u64,
    pub fees: Fees,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Fees {
    #[serde(deserialize_with = "from_str_to_u64")]
    pub in_bps: u64,
    #[serde(deserialize_with = "from_str_to_u64")]
    pub out_bps: u64,
}


#[derive(Deserialize)]
struct JsReturn {
    ok: bool,
    value: String,
}



pub async fn get_all_wrappers(factory_address: &str) -> Result<Vec<TokenWrapperInfo>, Box<dyn Error>> {
    js_try!(js_get_all_wrappers(factory_address) => Vec<TokenWrapperInfo>)
}


#[derive(Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct MetamaskInfo {
    pub address: String,
    #[serde(deserialize_with = "from_str_to_u32")]
    pub chain_id : u32,
}
// pub async fn connect_metamask(){// -> Result<(), Box<dyn Error>>{
//     let mut wallet = use_wallet();
//     wallet.is_connecting.set(true);
//     let meta_info : Result<MetamaskInfo, String> =  js_parse(js_connect_metamask().await); //js_try!(js_connect_metamask() => MetamaskInfo);
//     if let Ok(meta_info) = meta_info{
//         wallet.info.set(meta_info);
//     }
//     wallet.is_connecting.set(false);
// }

pub async fn connect_metamask()  -> Result<MetamaskInfo, Box<dyn Error>>{
        js_try!(js_connect_metamask() => MetamaskInfo)
}

pub async fn get_token_balance(user: &str, token: &str) -> Result<String,Box<dyn Error>>{
    js_try!(js_get_token_balance(user, token) => String)
}

pub async fn wrap_tokens(contract: &str, dToken: &str, amount: &str, cAsset: &str,) -> Result<String,Box<dyn Error>>{
    js_try!(js_wrap_tokens(contract, dToken, amount, cAsset) => String)
}

pub async fn unwrap_tokens(contract: &str, cAsset: &str, amount: &str, dToken: &str,) -> Result<String,Box<dyn Error>>{
    js_try!(js_unwrap_tokens(contract, cAsset, amount, dToken) => String)
}

// UniSwap
#[derive(Deserialize, Clone, Debug)]
pub struct PairInfo {
    pub token0: String,
    pub token1: String,
    pub pair_address: String,
    pub symbol0: Option<String>,
    pub symbol1: Option<String>,
    #[serde(deserialize_with = "from_str_to_opt_u64")]
    pub decimals0: Option<u64>,
    #[serde(deserialize_with = "from_str_to_opt_u64")]
    pub decimals1: Option<u64>,
    pub reserve0: Option<String>,
    pub reserve1: Option<String>,
}

pub async fn get_uniswap_v2_pairs(router_address: &str) -> Result<Vec<PairInfo>, String> {
    js_try!(js_get_uniswap_v2_pairs(router_address) => Vec<PairInfo>)
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct V3PoolState {
    pub sqrt_price_x96: String,
    #[serde(deserialize_with = "from_str_to_u128")]
    pub liquidity: u128,
}

pub async fn get_uniswap_v3_pool_states(pools: Vec<String>) -> Result<BTreeMap<String, V3PoolState>, String> {
    let js_array: Vec<JsValue> = pools.iter()
        .map(|s| JsValue::from_str(s))
        .collect();
    js_try!(js_get_uniswap_v3_pool_states(js_array) => BTreeMap<String, V3PoolState>)
}

pub async fn uniswap_v2_swap_tokens(
    token_in: &str,
    token_out: &str,
    amount_in: &str,
    amount_out_min: &str,
    router_address: &str,
) -> Result<String, String> {
    js_try!(js_swap_tokens(token_in,token_out,amount_in,amount_out_min,router_address) => String)
}
