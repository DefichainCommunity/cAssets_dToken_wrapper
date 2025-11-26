use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use std::collections::BTreeMap;
use serde::Deserialize;
use crate::js_try;
use super::{
    js_parse,
    deserialize::from_str_to_u128
};

#[wasm_bindgen(module = "/src/metamask.js")]
extern "C" {
    //uniswap v3
    async fn js_get_uniswap_v3_pool_states(pools: Vec<JsValue>) -> JsValue;
    async fn js_uniswap_v3_swap_tokens(token_in: &str, token_out: &str, amount_in: &str,
                            amount_out_min: &str, fee: &str, router_address: &str, is_native_in: bool, is_native_out: bool) -> JsValue;
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

pub async fn uniswap_v3_swap_tokens(
    token_in: &str,
    token_out: &str,
    amount_in: &str,
    amount_out_min: &str,
    fee: &str,
    router_address: &str,
    is_native_in: bool,
    is_native_out: bool,
) -> Result<String, String> {
    js_try!(js_uniswap_v3_swap_tokens(token_in,token_out,amount_in,amount_out_min,fee,router_address, is_native_in, is_native_out) => String)
}
