use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use std::error::Error;
use serde::Deserialize;
use crate::js_try;
use super::{
    js_parse,
    deserialize::from_str_to_u64
};

#[wasm_bindgen(module = "/src/metamask.js")]
extern "C" {
    // wrapper
    async fn js_get_all_wrappers(factory_address: &str) -> JsValue;
    async fn js_wrap_tokens(contract: &str, dToken: &str, amount: &str, cAsset: &str,) -> JsValue;
    async fn js_unwrap_tokens(contract: &str, cAsset: &str, amount: &str, dToken: &str) -> JsValue;
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

pub async fn get_all_wrappers(factory_address: &str) -> Result<Vec<TokenWrapperInfo>, Box<dyn Error>> {
    js_try!(js_get_all_wrappers(factory_address) => Vec<TokenWrapperInfo>)
}

pub async fn wrap_tokens(contract: &str, dToken: &str, amount: &str, cAsset: &str,) -> Result<String,Box<dyn Error>>{
    js_try!(js_wrap_tokens(contract, dToken, amount, cAsset) => String)
}

pub async fn unwrap_tokens(contract: &str, cAsset: &str, amount: &str, dToken: &str,) -> Result<String,Box<dyn Error>>{
    js_try!(js_unwrap_tokens(contract, cAsset, amount, dToken) => String)
}
