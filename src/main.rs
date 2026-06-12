use clap::Parser;
use semver::{Version, VersionReq};
use serde::Deserialize;
use toml::Table;
use ureq::Agent;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// on which project shall we SNIFF OUT THE SLOP??
    #[arg(value_name = "USER>/<REPO")]
    github_project: String,
}

#[derive(Deserialize, Debug)]
struct CargoToml {
    workspace: Option<Workspace>,
    package: Option<Package>,
    dependencies: Option<Dependencies>,
}

type Dependencies = Table;

#[derive(Deserialize, Debug)]
struct Workspace {
    resolver: Option<String>,
    package: Option<Package>,
    dependencies: Option<Dependencies>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct Package {
    edition: Option<String>,
    // rust_version: Option<String>, // TODO:
}

#[derive(Deserialize, Debug)]
struct RegistryVersionsResponse {
    // TODO: stop allocing the entire vec.. just parse out the latest version, nothign more...
    // maybe even get rid of this serde struct thingy, just rawdog it?
    versions: Vec<RegistryCrateVersion>,
}

#[derive(Deserialize, Debug)]
struct RegistryCrateVersion {
    num: String,
}

fn is_old_edition(edition_str: &str) -> bool {
    edition_str.parse::<u16>().unwrap() < 2024
}

fn find_outdated_dependencies(
    dependencies: Dependencies,
    num_outdated_dependencies: &mut u16,
    agent: &Agent,
) -> color_eyre::Result<()> {
    println!("checking for outdated dependencies");

    for (crate_name, value) in dependencies {
        // FIXME: fucking stop cloning
        let version_str = match &value {
            toml::Value::Table(table) if let Some(version_entry) = table.get("version") => {
                match version_entry {
                    toml::Value::String(version_str) => version_str.clone(),
                    _ => unreachable!("we are fucked"),
                }
            }
            toml::Value::String(version_str) => version_str.clone(),
            _ => continue,
        };
        let version_req = VersionReq::parse(&version_str)?;

        let versions_response: RegistryVersionsResponse = agent
            .get(format!(
                "https://crates.io/api/v1/crates/{}/versions",
                crate_name
            ))
            .call()?
            .body_mut()
            .read_json()?;

        let latest_version: Version = versions_response.versions[0].num.parse()?;

        if !version_req.matches(&latest_version) {
            println!(
                "- {}: using {} but latest is {}",
                crate_name, version_req, latest_version
            );
            *num_outdated_dependencies += 1;
        }
    }

    Ok(())
}

fn find_ai_string_in_gitignore(gitignore: &str) -> bool {
    println!("checking for AI related strings in .gitignore");
    for line in gitignore.lines() {
        if line.to_ascii_lowercase().contains("claude") {
            return true
        }
        if line.to_ascii_lowercase().contains("agents") {
            return true
        }
        if line.to_ascii_lowercase().contains("copilot-instructions") {
            return true
        }
        if line.to_ascii_lowercase().contains("cursor/rules") {
            return true
        }
        if line.to_ascii_lowercase().contains("codex/rules") {
            return true
        }
    }
    false
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let cargo_toml_raw_url = format!(
        "https://raw.githubusercontent.com/{}/HEAD/Cargo.toml",
        args.github_project
    );

    let agent_md_raw_url = format!(
        "https://raw.githubusercontent.com/{}/HEAD/AGENTS.md",
        args.github_project
    );

    let gitignore_raw_url = format!(
        "https://raw.githubusercontent.com/{}/HEAD/.gitignore",
        args.github_project
    );

    let agent: Agent = Agent::config_builder()
        .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
        .build()
        .into();

    let cargo_toml_str = agent
        .get(cargo_toml_raw_url)
        .call()?
        .body_mut()
        .read_to_string()?;

    let cargo_toml: CargoToml = toml::from_str(&cargo_toml_str)?;

    let mut slop_score = 0;
    let mut slop_score_motivations = Vec::new();
    let mut num_outdated_dependencies = 0;
    let mut has_agents_md = false;
    let mut has_agents_md_in_gitignore = false;

    if let Some(package) = cargo_toml.package
        && let Some(edition) = package.edition
        && is_old_edition(&edition)
    {
        slop_score += 1;
        slop_score_motivations.push(format!("using old Rust edition ({})", edition));
    }

    if let Some(workspace) = cargo_toml.workspace {
        if let Some(package) = workspace.package
            && let Some(edition) = package.edition
            && is_old_edition(&edition)
        {
            slop_score += 1;
            slop_score_motivations.push(format!("using old Rust edition ({})", edition));
        }

        if let Some(resolver) = workspace.resolver
            && resolver.parse::<u8>().unwrap() < 3
        {
            slop_score += 1;
            slop_score_motivations.push(format!("using old workspace resolver ({})", resolver));
        }

        if let Some(dependencies) = workspace.dependencies {
            find_outdated_dependencies(dependencies, &mut num_outdated_dependencies, &agent)?;
        }
    }

    if let Some(dependencies) = cargo_toml.dependencies {
        find_outdated_dependencies(dependencies, &mut num_outdated_dependencies, &agent)?;
    }

    slop_score += num_outdated_dependencies;

    match agent.get(agent_md_raw_url).call() {
        Ok(_resp) => {
            slop_score += 100;
            has_agents_md = true;
        },
        Err(_e) => {},
    }

    match agent.get(gitignore_raw_url).call() {
        Ok(mut resp) => {
            let gitignore_str = resp.body_mut().read_to_string()?;
            has_agents_md_in_gitignore = find_ai_string_in_gitignore(&gitignore_str);
            if has_agents_md_in_gitignore {
                slop_score += 200;
            }
        }
        Err(_e) => {}
    }

        

    println!("\nslop score: {}", slop_score);

    for motivation in slop_score_motivations {
        println!("- {}", motivation);
    }
    if num_outdated_dependencies > 0 {
        println!(
            "- using {} outdated dependencies",
            num_outdated_dependencies
        );
    }
    if has_agents_md {
        println!("- has an AGENTS.md file");
    }
    if has_agents_md_in_gitignore {
        println!("- has an AGENTS.md or CLAUDE.md file in .gitignore");
    }

    Ok(())
}
