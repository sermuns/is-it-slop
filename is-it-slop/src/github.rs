use color_eyre::eyre::Context;
use futures::stream::{self, StreamExt};
use jiff::Timestamp;
use reqwest::{Client, Url};
use serde::Deserialize;

use crate::GitHubProject;

#[derive(Deserialize, Debug)]
pub struct GithubRepoDetails {
    pub created_at: Timestamp,
}

#[derive(Deserialize, Debug)]
struct GithubCommitEntry {
    commit: GithubCommit,
}

#[derive(Deserialize, Debug)]
struct GithubCommit {
    message: String,
}

static SUSSY_FILES: &[&str] = &[
    "AGENTS.md",
    "CLAUDE.md",
    ".github/copilot-instructions.md",
    ".cursor/rules",
    ".codex/rules",
    ".hermes/soul",
];

static SUSSY_COAUTHOR_PATTERNS: &[&str] = &[
    "claude <noreply@anthropic.com>",
    "claude code <noreply@anthropic.com>",
    "cursoragent@cursor.com>",
    "+copilot@users.noreply.github.com>",
    "<copilot@users.noreply.github.com>",
    "+devin-ai-integration[bot]@users.noreply.github.com>",
];

pub async fn fetch_repo_details(
    github_project: &GitHubProject,
    client: &Client,
) -> color_eyre::Result<GithubRepoDetails> {
    client
        .get(format!(
            "https://api.github.com/repos/{}/{}",
            github_project.owner, github_project.repo
        ))
        .send()
        .await
        .wrap_err("couldn't fetch repo details, are you sure it exists?")?
        .json()
        .await
        .map_err(color_eyre::Report::from)
}

pub async fn find_sussy_coauthors(
    github_project: &GitHubProject,
    git_ref: &str,
    client: &Client,
) -> Vec<String> {
    println!("\nchecking recent commits for sussy co-authors");

    let commits_url = Url::parse_with_params(
        &format!(
            "https://api.github.com/repos/{}/{}/commits",
            github_project.owner, github_project.repo
        ),
        &[("sha", git_ref), ("per_page", "100")],
    )
    .expect("github commits URL should be valid");

    let commits: Vec<GithubCommitEntry> = match client.get(commits_url).send().await {
        Ok(response) => match response.json().await {
            Ok(commits) => commits,
            Err(e) => {
                println!("error while reading commit list: {:?}", e);
                return Vec::new();
            }
        },
        Err(e) => {
            println!("error while fetching commit list: {:?}", e);
            return Vec::new();
        }
    };

    let mut matches = Vec::new();

    for commit in commits {
        for line in commit.commit.message.lines() {
            let Some((_, coauthor)) = line.split_once(':') else {
                continue;
            };

            if !line
                .get(.."co-authored-by".len())
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case("co-authored-by"))
            {
                continue;
            }

            let coauthor = coauthor.trim();
            if is_sussy_coauthor(coauthor) && !matches.iter().any(|seen| seen == coauthor) {
                matches.push(coauthor.to_owned());
            }
        }
    }

    matches
}

fn is_sussy_coauthor(coauthor: &str) -> bool {
    let coauthor = coauthor.to_ascii_lowercase();

    SUSSY_COAUTHOR_PATTERNS
        .iter()
        .any(|pattern| coauthor.contains(pattern))
}

pub async fn find_sussy_files(
    github_project: &GitHubProject,
    git_ref: &str,
    client: &Client,
) -> Vec<String> {
    println!("\nchecking for sussy files in the repo");

    stream::iter(SUSSY_FILES)
        .map(|sussy_file| async {
            client
                .get(format_raw_github_file_url(
                    github_project,
                    git_ref,
                    sussy_file,
                ))
                .send()
                .await
                .ok()?
                .error_for_status()
                .is_ok()
                .then_some(sussy_file.to_string())
        })
        .buffer_unordered(20)
        .filter_map(|f| async { f })
        .collect()
        .await
}

pub async fn fetch_gitignore(
    github_project: &GitHubProject,
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

pub fn format_raw_github_file_url(
    github_project: &GitHubProject,
    git_ref: &str,
    path: &str,
) -> String {
    format!(
        "https://raw.githubusercontent.com/{owner}/{repo}/{git_ref}/{path}",
        owner = github_project.owner,
        repo = github_project.repo,
    )
}
