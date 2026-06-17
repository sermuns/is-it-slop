use dioxus::prelude::*;

pub const PKG_NAME: &str = "is-it-slop";

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        Stylesheet { href: asset!("/style.css") }

        h1 {{PKG_NAME}}
    }
}
