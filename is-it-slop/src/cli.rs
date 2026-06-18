use std::str::FromStr;

use clap::Parser;
use color_eyre::eyre::{OptionExt, bail};
use reqwest::Url;

#[derive(Parser, Debug)]
#[command(version, about, arg_required_else_help = true)]
pub struct Args {
    /// Either <USER>/<REPO> or full URL
    pub github_project_or_url: GitHubProject,

    /// Emit non-zero exit code if any slop detected
    #[arg(long)]
    pub check: bool,

    /// HEAD is a reasonable standard, but you can manually specify branch name or specific commit
    #[arg(long, default_value = "HEAD")]
    pub git_ref: String,
}

#[derive(Debug, Clone)]
pub struct GitHubProject {
    pub owner: String,
    pub repo: String,
    pub url: Option<Url>,
}

impl FromStr for GitHubProject {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(url) = Url::parse(s) {
            if url.host_str() != Some("github.com") {
                bail!("not a GitHub URL!");
            }
            let mut segments = url
                .path_segments()
                .ok_or_eyre("cannot parse cannot-be-a-base URL")?;
            let (Some(owner), Some(repo), None) =
                (segments.next(), segments.next(), segments.next())
            else {
                bail!("path segments do not match format '/<owner>/<repository>'");
            };
            return Ok(GitHubProject {
                owner: owner.to_owned(),
                repo: repo.to_owned(),
                url: Some(url),
            });
        }

        let mut segments = s.split('/');
        let (Some(owner), Some(repo), None) = (segments.next(), segments.next(), segments.next())
        else {
            bail!("argument does not match format '<owner>/<repository>'");
        };

        Ok(GitHubProject {
            owner: owner.to_owned(),
            repo: repo.to_owned(),
            url: None,
        })
    }
}
