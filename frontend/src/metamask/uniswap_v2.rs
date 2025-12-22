use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use std::error::Error;
use serde::Deserialize;
use crate::js_try;
use super::{
    js_parse,
    deserialize::from_str_to_opt_u64
};

#[wasm_bindgen(module = "/src/metamask.js")]
extern "C" {
    // uniswap v2
    async fn js_get_uniswap_v2_pairs(router_address: &str) -> JsValue;
    async fn js_uniswap_v2_swap_tokens(token_in: &str, token_out: &str, amount_in: &str,
                                       amount_out_min: &str, router_address: &str, is_native_in: bool, is_native_out: bool) -> JsValue;
    async fn js_uniswap_v2_add_liquidity(token_a: &str, token_b: &str, amount_a: &str, amount_b: &str, router_address: &str, is_native_a: bool, is_native_b: bool) -> JsValue;
}


// UniSwap
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct V2PairInfo {
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

pub async fn get_uniswap_v2_pairs(router_address: &str) -> Result<Vec<V2PairInfo>, String> {
    js_try!(js_get_uniswap_v2_pairs(router_address) => Vec<V2PairInfo>)
}

pub async fn uniswap_v2_swap_tokens(
    token_in: &str,
    token_out: &str,
    amount_in: &str,
    amount_out_min: &str,
    router_address: &str,
    is_native_in: bool,
    is_native_out: bool,
) -> Result<String, String> {
    js_try!(js_uniswap_v2_swap_tokens(token_in,token_out,amount_in,amount_out_min,router_address, is_native_in, is_native_out) => String)
}

pub async fn  uniswap_v2_add_liquidity(
    token_a: &str,
    token_b: &str,
    amount_a: &str,
    amount_b: &str,
    router_address: &str,
    is_native_a: bool,
    is_native_b: bool,
)-> Result<String, String> {
    js_try!(js_uniswap_v2_add_liquidity(token_a,token_b,amount_a,amount_b,router_address, is_native_a, is_native_b) => String)
}
