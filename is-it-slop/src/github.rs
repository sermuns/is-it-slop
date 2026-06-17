use color_eyre::eyre::Context;
use futures::stream::{self, StreamExt};
use jiff::Timestamp;
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GithubRepoDetails {
    pub created_at: Timestamp,
}

static SUSSY_FILES: &[&str] = &[
    "AGENTS.md",
    "CLAUDE.md",
    ".github/copilot-instructions.md",
    ".cursor/rules",
    ".codex/rules",
    ".hermes/soul",
];

pub async fn fetch_repo_details(
    github_project: &str,
    client: &Client,
) -> color_eyre::Result<GithubRepoDetails> {
    client
        .get(String::from("https://api.github.com/repos/") + github_project)
        .send()
        .await
        .wrap_err("couldn't fetch repo details, are you sure it exists?")?
        .json()
        .await
        .map_err(color_eyre::Report::from)
}

pub async fn find_sussy_files(github_project: &str, git_ref: &str, client: &Client) -> Vec<String> {
    println!("\nchecking for sussy files in the repo");

    stream::iter(SUSSY_FILES)
        .filter_map(|sussy_file| async {
            client
                .get(format_raw_github_file_url(
                    github_project,
                    git_ref,
                    sussy_file,
                ))
                .send()
                .await
                .is_ok()
                .then_some(sussy_file.to_string())
        })
        .collect()
        .await
}

pub async fn fetch_gitignore(
    github_project: &str,
    git_ref: &str,
    client: &Client,
) -> color_eyre::Result<String> {
    client
        .get(format_raw_github_file_url(
            github_project,
            git_ref,
            ".gitignore",
        ))
        .send()
        .await?
        .text()
        .await
        .map_err(color_eyre::Report::from)
}

pub fn find_gitignored_sussy_files(gitignore: &str) -> Vec<&str> {
    println!("\nchecking for sussy files in .gitignore");

    SUSSY_FILES
        .iter()
        .filter_map(|sussy_file| gitignore.matches(sussy_file).next())
        .collect()
}

pub fn format_raw_github_file_url(github_project: &str, git_ref: &str, path: &str) -> String {
    format!(
        "https://raw.githubusercontent.com/{}/{}/{}",
        github_project, git_ref, path
    )
}
