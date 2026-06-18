use std::str::FromStr;

use dioxus::prelude::*;
use is_it_slop::{SlopReport, generate_slop_report, github::GitHubProject};

pub const PKG_NAME: &str = "is-it-slop";

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut project_search_input = use_signal(String::new);
    let mut report_text = use_signal(String::new);

    let fetch = move |_| async move {
        let github_project = match GitHubProject::from_str(&project_search_input()) {
            Ok(p) => p,
            Err(e) => {
                report_text.set(e.to_string());
                return;
            }
        };
        let SlopReport { text, .. } = /*match */generate_slop_report(&github_project, "HEAD").await.unwrap();
        report_text.set(text);
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

        p { {report_text} }
    }
}
