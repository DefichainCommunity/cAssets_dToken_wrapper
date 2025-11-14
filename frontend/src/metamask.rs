use std::error::Error;
use wasm_bindgen::prelude::*;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_wasm_bindgen::from_value;

// Bind JS functions in metamask.js
#[wasm_bindgen(module = "/src/metamask.js")]
extern "C" {
    pub async fn js_connect_metamask() -> JsValue;
    pub async fn js_get_token_balance(user: &str, token: &str) -> JsValue;
    pub async fn js_get_all_wrappers(factory_address: &str) -> JsValue;
    pub async fn js_wrap_tokens(contract: &str, dToken: &str, amount: &str, cAsset: &str,) -> JsValue;
    pub async fn js_unwrap_tokens(contract: &str, cAsset: &str, amount: &str, dToken: &str) -> JsValue;
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

fn from_str_to_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    s.parse::<u64>().map_err(serde::de::Error::custom)
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

pub async fn connect_metamask() -> Result<String, Box<dyn Error>>{
    js_try!(js_connect_metamask() => String)
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
