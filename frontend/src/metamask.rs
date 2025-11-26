use std::error::Error;
use wasm_bindgen::prelude::*;
use serde::Deserialize;
use serde::de::DeserializeOwned;

mod deserialize;
pub mod wrapper;
pub mod uniswap_v2;
pub mod uniswap_v3;

use deserialize::*;
// Bind JS functions in metamask.js
#[wasm_bindgen(module = "/src/metamask.js")]
extern "C" {
    async fn js_connect_metamask() -> JsValue;
    pub fn js_on_chain_changed(callback: &Closure<dyn FnMut(u32)>);
    pub fn js_on_accounts_changed(callback: &Closure<dyn FnMut(Vec<JsValue>)>);
    async fn js_get_token_balance(user: &str, token: &str, is_native: bool) -> JsValue;
}

pub fn js_parse<T: DeserializeOwned>(js: JsValue) -> Result<T, String> {
    // Parse the wrapper
    let wrapper: JsReturn = serde_wasm_bindgen::from_value(js).map_err(|e| format!("{:?}", e))?;
    if !wrapper.ok { return Err(wrapper.value);} // This is already a string
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


#[derive(Deserialize)]
struct JsReturn {
    ok: bool,
    value: String,
}

#[derive(Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct MetamaskInfo {
    pub address: String,
    #[serde(deserialize_with = "from_str_to_u32")]
    pub chain_id : u32,
}

pub async fn connect_metamask()  -> Result<MetamaskInfo, Box<dyn Error>>{
        js_try!(js_connect_metamask() => MetamaskInfo)
}

pub async fn get_token_balance(user: &str, token: &str, is_native: bool) -> Result<String,Box<dyn Error>>{
    js_try!(js_get_token_balance(user, token, is_native) => String)
}
