use std::str::FromStr;

use dioxus::prelude::*;
use is_it_slop::{
    github::{GitHubProject, fetch_gitignore},
    reqwest,
};

pub const PKG_NAME: &str = "is-it-slop";

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut project_search_input = use_signal(String::new);
    let mut contents = use_signal(String::new);

    let fetch = move |_| async move {
        info!("{}", project_search_input());
        let project = GitHubProject::from_str(&project_search_input()).unwrap();
        let client = reqwest::Client::new();
        info!("{:?}", project);
        let gitignore = fetch_gitignore(&project, "HEAD", &client).await.unwrap();
        contents.set(gitignore);
    };

    rsx! {
        Stylesheet { href: asset!("/style.css") }

        h1 { {PKG_NAME} }

        input {
            onchange: move |e| project_search_input.set(e.value())
        }

        button {
            onclick: fetch,
            "hello this is sick!"
        }

        code { "{contents}" }
    }
}
