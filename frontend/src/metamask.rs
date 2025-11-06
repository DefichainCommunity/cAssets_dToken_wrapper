use wasm_bindgen::prelude::*;

// Bind JS functions in metamask.js
#[wasm_bindgen(module = "/src/metamask.js")]
extern "C" {
    pub async fn connect_metamask() -> JsValue;
    pub async fn wrap_tokens(contract: &str, dToken: &str, amount: &str, cAsset: &str,) -> JsValue;
    pub async fn unwrap_tokens(contract: &str, cAsset: &str, amount: &str, dToken: &str) -> JsValue;
}
