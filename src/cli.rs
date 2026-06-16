use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, arg_required_else_help = true)]
pub struct Args {
    /// Either <USER>/<REPO> or full URL
    pub github_project_or_url: String,

    /// Emit non-zero exit code if any slop detected
    #[arg(long)]
    pub check: bool,

    /// HEAD is a reasonable standard, but you can manually specify branch name or specific commit
    #[arg(long, default_value = "HEAD")]
    pub git_ref: String,
}
