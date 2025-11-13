mod app;
mod metamask;

fn main() {
    // Launch the root component
    console_log::init_with_level(log::Level::Debug).expect("failed to init logger");
    dioxus::launch(app::App);
}
