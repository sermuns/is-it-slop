use color_eyre::eyre::Context;
use futures::prelude::*;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use semver::{Version, VersionReq};
use serde::Deserialize;
use toml::Table;

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

pub async fn look_for_outdated_dependencies(
    dependencies: Dependencies,
    client: &Client,
) -> Vec<String> {
    println!("\nlooking for outdated dependencies");

    let pb = ProgressBar::new(dependencies.len() as u64)
        .with_style(ProgressStyle::with_template("{msg} {wide_bar} {pos}/{len}").unwrap());

    // TODO: don't just silently drop entries on error!
    stream::iter(dependencies)
        .map(|(crate_name, value)| {
            let pb = pb.clone();

            async move {
                let version_str = match &value {
                    toml::Value::Table(table) if let Some(version_entry) = table.get("version") => {
                        match version_entry {
                            toml::Value::String(version_str) => version_str,
                            _ => unreachable!("we are fucked"),
                        }
                    }
                    toml::Value::String(version_str) => version_str,
                    _ => return None,
                };
                let version_req = VersionReq::parse(version_str).ok()?;

                let registry_crate_response: RegistryCrateMetadata = client
                    .get(format!("https://crates.io/api/v1/crates/{}", crate_name))
                    .send()
                    .await
                    .ok()?
                    .json()
                    .await
                    .ok()?;

                let latest_stable_version = registry_crate_response.krate.max_stable_version;

                pb.inc(1);
                pb.set_message(crate_name.clone()); // TODO: maybe avoid cloning

                if version_req.matches(&latest_stable_version) {
                    None
                } else {
                    Some(format!(
                        "- {}: using {} but latest stable is {}",
                        crate_name, version_req, latest_stable_version
                    ))
                }
            }
        })
        .buffer_unordered(20)
        .filter_map(|x| async { x }) // kinda shitty
        .collect()
        .await
}

pub fn is_old_edition(edition_str: &str) -> color_eyre::Result<bool> {
    Ok(edition_str
        .parse::<u16>()
        .wrap_err("edition wasn't possible to parse as a year")?
        < 2024)
}

pub async fn fetch_cargo_toml(
    github_project: &str,
    git_ref: &str,
    client: &Client,
) -> color_eyre::Result<CargoToml> {
    let cargo_toml_str = client
        .get(format!(
            "https://raw.githubusercontent.com/{}/{}/Cargo.toml",
            github_project, git_ref,
        ))
        .send()
        .await
        .wrap_err("no `Cargo.toml` present in root of repo")?
        .text()
        .await?;

    toml::from_str(&cargo_toml_str).map_err(color_eyre::Report::from)
}
