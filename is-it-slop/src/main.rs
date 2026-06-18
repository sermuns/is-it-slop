use clap::Parser;
use is_it_slop::{SlopReport, generate_slop_report};

mod cli;

use crate::cli::Args;

#[allow(clippy::too_many_lines)]
#[tokio::main(flavor = "current_thread")]
async fn main() -> color_eyre::Result<()> {
    color_eyre::config::HookBuilder::default()
        .display_env_section(false)
        .display_location_section(cfg!(debug_assertions))
        .install()?;

    let args = Args::parse();
    let github_project = args.github_project_or_url;

    let SlopReport { slop_score, .. } =
        generate_slop_report(&github_project, &args.git_ref).await?;

    if args.check && slop_score > 0 {
        std::process::exit(1);
    }

    Ok(())
}
