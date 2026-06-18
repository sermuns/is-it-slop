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
    let mut contents = use_signal(String::new);

    let fetch = move |_| async move {
        let project = GitHubProject {
            repo: "ampy".into(),
            owner: "HSScodes".into(),
            url: None,
        };
        let client = reqwest::Client::new();
        let gitignore = fetch_gitignore(&project, "HEAD", &client).await.unwrap();
        contents.set(gitignore);
    };

    rsx! {
        Stylesheet { href: asset!("/style.css") }

        h1 { {PKG_NAME} }

        button { onclick: fetch }

        code { "{contents}" }
    }
}
