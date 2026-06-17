use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        Stylesheet { href: asset!("/style.css") }

        {env!("CARGO_PKG_NAME")}
    }
}
