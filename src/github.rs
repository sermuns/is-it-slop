use color_eyre::eyre::Context;
use jiff::Timestamp;
use serde::Deserialize;
use ureq::Agent;

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

pub fn fetch_repo_details(
    github_project: &str,
    agent: &Agent,
) -> color_eyre::Result<GithubRepoDetails> {
    agent
        .get(String::from("https://api.github.com/repos/") + github_project)
        .call()
        .wrap_err("couldn't fetch repo details, are you sure it exists?")?
        .body_mut()
        .read_json()
        .map_err(color_eyre::Report::from)
}

pub fn find_sussy_files(github_project: &str, git_ref: &str, agent: &Agent) -> Vec<String> {
    SUSSY_FILES
        .iter()
        .filter_map(|sussy_file| {
            agent
                .get(format_raw_github_file_url(
                    github_project,
                    git_ref,
                    sussy_file,
                ))
                .call()
                .is_ok()
                .then_some(sussy_file.to_string())
        })
        .collect()
}

pub fn fetch_gitignore(github_project: &str, agent: &Agent) -> color_eyre::Result<String> {
    Ok(agent
        .get(format!(
            "https://raw.githubusercontent.com/{}/HEAD/.gitignore",
            github_project
        ))
        .call()?
        .body_mut()
        .read_to_string()?)
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
