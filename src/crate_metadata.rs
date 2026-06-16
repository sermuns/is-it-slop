use color_eyre::eyre::Context;
use semver::{Version, VersionReq};
use serde::Deserialize;
use toml::Table;
use ureq::Agent;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct CargoToml {
    pub workspace: Option<Workspace>,
    pub package: Option<Package>,
    pub dependencies: Option<Dependencies>,
    pub dev_dependencies: Option<Dependencies>,
    pub build_dependencies: Option<Dependencies>,
}

type Dependencies = Table;

#[derive(Deserialize, Debug)]
pub struct Workspace {
    pub resolver: Option<String>,
    pub package: Option<Package>,
    pub dependencies: Option<Dependencies>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Package {
    pub edition: Option<String>,
    // rust_version: Option<String>, // TODO:
}

#[derive(Deserialize, Debug)]
struct RegistryCrateMetadata {
    #[serde(rename = "crate")]
    krate: RegistryCrate, // `crate` is reserved keyword
}

#[derive(Deserialize, Debug)]
struct RegistryCrate {
    max_stable_version: Version,
}

// TODO: do the requests concurrently
// it's embarrassingly parallel....
// maybe switch to reqwest and have a tokio runtime..
pub fn look_for_outdated_dependencies(
    dependencies: Dependencies,
    num_outdated_dependencies: &mut u16,
    agent: &Agent,
) -> color_eyre::Result<()> {
    println!("\nlooking for outdated dependencies");

    for (crate_name, value) in dependencies {
        let version_str = match &value {
            toml::Value::Table(table) if let Some(version_entry) = table.get("version") => {
                match version_entry {
                    toml::Value::String(version_str) => version_str,
                    _ => unreachable!("we are fucked"),
                }
            }
            toml::Value::String(version_str) => version_str,
            _ => continue,
        };
        let version_req = VersionReq::parse(version_str)?;

        let registry_crate_response: RegistryCrateMetadata = agent
            .get(format!("https://crates.io/api/v1/crates/{}", crate_name))
            .call()?
            .body_mut()
            .read_json()?;

        let latest_stable_version = registry_crate_response.krate.max_stable_version;

        if !version_req.matches(&latest_stable_version) {
            println!(
                "- {}: using {} but latest stable is {}",
                crate_name, version_req, latest_stable_version
            );
            *num_outdated_dependencies += 1;
        }
    }

    Ok(())
}

pub fn is_old_edition(edition_str: &str) -> color_eyre::Result<bool> {
    Ok(edition_str
        .parse::<u16>()
        .wrap_err("edition wasn't possible to parse as a year")?
        < 2024)
}

pub fn fetch_cargo_toml(
    github_project: &str,
    git_ref: &str,
    agent: &Agent,
) -> color_eyre::Result<CargoToml> {
    let cargo_toml_str = agent
        .get(format!(
            "https://raw.githubusercontent.com/{}/{}/Cargo.toml",
            github_project, git_ref,
        ))
        .call()
        .wrap_err("no `Cargo.toml` present in root of repo")?
        .body_mut()
        .read_to_string()?;

    toml::from_str(&cargo_toml_str).map_err(color_eyre::Report::from)
}
