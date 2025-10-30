use wasm_bindgen::prelude::*;

// Bind JS functions in metamask.js
#[wasm_bindgen(module = "/src/metamask.js")]
extern "C" {
    pub async fn connect_metamask() -> JsValue;
    pub async fn wrap_tokens(contract: &str, underlying: &str, amount: &str) -> JsValue;
    pub async fn unwrap_tokens(contract: &str, wrapped: &str, amount: &str) -> JsValue;
}
