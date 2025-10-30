mod app;
mod metamask;

fn main() {
    // Launch the root component
    dioxus::launch(app::App);
}
