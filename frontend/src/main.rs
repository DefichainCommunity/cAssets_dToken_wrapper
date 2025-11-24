mod app;
mod components;
mod metamask;
mod vanillaswap;
mod wrapper;
mod wallet_context;

fn main() {
    // Launch the root component
    console_log::init_with_level(log::Level::Debug).expect("failed to init logger");
    dioxus::launch(app::App);
}
