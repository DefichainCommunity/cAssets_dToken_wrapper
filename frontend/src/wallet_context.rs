use dioxus::prelude::*;
use crate::metamask::MetamaskInfo;

#[derive(Clone)]
pub struct WalletContext {
    pub info: Signal<MetamaskInfo>,
    pub is_connecting: Signal<bool>,
}

pub fn use_wallet() -> WalletContext {
    use_context::<WalletContext>()
}
