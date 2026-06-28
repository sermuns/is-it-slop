use clap::Parser;
use jiff::{Unit, ZonedDifference, tz::TimeZone};
use reqwest::Client;

mod cli;
mod crate_metadata;
mod github;

pub use crate::cli::GitHubProject;
use crate::{
    cli::Args,
    crate_metadata::{fetch_cargo_toml, is_old_edition, look_for_outdated_dependencies},
    github::{
        fetch_gitignore, fetch_repo_details, find_gitignored_sussy_files, find_sussy_coauthors,
        find_sussy_files,
    },
};

pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[allow(clippy::too_many_lines)]
#[tokio::main(flavor = "current_thread")]
async fn main() -> color_eyre::Result<()> {
    color_eyre::config::HookBuilder::default()
        .display_env_section(false)
        .display_location_section(cfg!(debug_assertions))
        .install()?;

    let args = Args::parse();
    let github_project = args.github_project_or_url;

    let client = Client::builder().user_agent(USER_AGENT).build()?;

    let mut slop_score_motivations = Vec::new();

    println!(
        "checking 'https://github.com/{}/{}'",
        github_project.owner, github_project.repo
    );

    let repo = fetch_repo_details(&github_project, &client).await?;

    let now_utc = jiff::Timestamp::now().to_zoned(TimeZone::UTC);
    let created_utc = repo.created_at.to_zoned(TimeZone::UTC);

    let duration_since_creation = now_utc.since(
        ZonedDifference::new(&created_utc)
            .smallest(jiff::Unit::Hour)
            .largest(jiff::Unit::Year),
    )?;

    println!("it is {:#} old", duration_since_creation);

    const YOUNG_AGE_HOURS: f64 = 24.;
    match duration_since_creation.total((Unit::Hour, &now_utc)) {
        Ok(duration_in_hours) if duration_in_hours < YOUNG_AGE_HOURS => {
            slop_score_motivations.push(format!(
                "the repo is younger than {} hours",
                YOUNG_AGE_HOURS
            ));
        }
        Err(e) => {
            println!("error during age calculation: {:?}", e);
        }
        _ => (),
    }

    let cargo_toml = fetch_cargo_toml(&github_project, &args.git_ref, &client).await?;

    if let Some(package) = cargo_toml.package
        && let Some(edition) = package.edition
        && is_old_edition(&edition)?
    {
        slop_score_motivations.push(format!("using old Rust edition ({})", edition));
    }

    let mut outdated_dependencies = Vec::new();
    let has_dependencies = cargo_toml.dependencies.is_some()
        || cargo_toml.dev_dependencies.is_some()
        || cargo_toml.build_dependencies.is_some()
        || cargo_toml
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.dependencies.as_ref())
            .is_some();

    if has_dependencies {
        println!("\nlooking for outdated dependencies");
    }

    if let Some(dependencies) = cargo_toml.dependencies {
        outdated_dependencies.extend(look_for_outdated_dependencies(dependencies, &client).await);
    }
    if let Some(dev_dependencies) = cargo_toml.dev_dependencies {
        outdated_dependencies
            .extend(look_for_outdated_dependencies(dev_dependencies, &client).await);
    }
    if let Some(build_dependencies) = cargo_toml.build_dependencies {
        outdated_dependencies
            .extend(look_for_outdated_dependencies(build_dependencies, &client).await);
    }

    if let Some(workspace) = cargo_toml.workspace {
        if let Some(package) = workspace.package
            && let Some(edition) = package.edition
            && is_old_edition(&edition)?
        {
            slop_score_motivations.push(format!("using old Rust edition ({})", edition));
        }

        if let Some(resolver) = workspace.resolver
            && resolver.parse::<u8>().unwrap() < 3
        {
            slop_score_motivations.push(format!("using old workspace resolver ({})", resolver));
        }

        if let Some(dependencies) = workspace.dependencies {
            outdated_dependencies
                .extend(look_for_outdated_dependencies(dependencies, &client).await);
        }
    }

    let sussy_files_present = find_sussy_files(&github_project, &args.git_ref, &client).await;
    let sussy_coauthors = find_sussy_coauthors(&github_project, &args.git_ref, &client).await;

    let gitignore = fetch_gitignore(&github_project, &args.git_ref, &client).await?;
    let sussy_files_gitignored = find_gitignored_sussy_files(&gitignore);

    let slop_score = outdated_dependencies.len()
        + slop_score_motivations.len()
        + sussy_files_present.len()
        + sussy_files_gitignored.len()
        + sussy_coauthors.len();

    println!("\nslop score: {}", slop_score);

    for motivation in slop_score_motivations {
        println!("- {}", motivation);
    }

    if !outdated_dependencies.is_empty() {
        println!("- outdated dependencies");
        for outdated_dependency_motivation in outdated_dependencies {
            println!("  - {}", outdated_dependency_motivation);
        }
    }
    if !sussy_files_present.is_empty() {
        println!("- sussy files present:");
        for sussy_file in sussy_files_present {
            println!("  - {}", sussy_file);
        }
    }
    if !sussy_files_gitignored.is_empty() {
        println!("- sussy files gitignored:");
        for sussy_file in sussy_files_gitignored {
            println!("  - {}", sussy_file);
        }
    }
    if !sussy_coauthors.is_empty() {
        println!("- sussy co-authors:");
        for coauthor in sussy_coauthors {
            println!("  - {}", coauthor);
        }
    }

    if args.check && slop_score > 0 {
        std::process::exit(1);
    }

    Ok(())
}
