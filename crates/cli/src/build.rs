use anyhow::Context;
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

#[derive(Deserialize, Debug)]
struct RootCargo {
    workspace: Workspace,
}

#[derive(Deserialize, Debug)]
struct Workspace {
    members: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct CrateCargo {
    package: Package,
}

#[derive(Deserialize, Debug)]
struct Package {
    name: String,
}

pub fn build(extension: Option<PathBuf>, out: PathBuf, release: bool) -> anyhow::Result<()> {
    match extension {
        Some(path) => {
            build_extension(&path.as_os_str().to_string_lossy(), &out, release)?;
        }
        None => {
            let members = {
                let content =
                    fs::read_to_string("Cargo.toml").context("unable to open 'Cargo.toml'")?;
                let cargo = toml::from_str::<RootCargo>(&content)
                    .context("failed to parse workspace members from 'Cargo.toml'")?;
                cargo.workspace.members
            };

            let extensions = members.iter().filter(|v| v.starts_with("extensions/"));

            for extension in extensions {
                build_extension(extension, &out, release)?
            }
        }
    }

    Ok(())
}

fn build_extension(path: &str, out: &Path, release: bool) -> anyhow::Result<()> {
    let package_name = {
        let path = Path::new(path).join("Cargo.toml");
        let content = fs::read_to_string(path)?;
        let cargo = toml::from_str::<CrateCargo>(&content)?;
        cargo.package.name
    };

    let mut args = vec![
        "build",
        "-p",
        &package_name,
        "--target",
        "wasm32-unknown-unknown",
    ];

    if release {
        args.push("--release");
    }

    let mut command = Command::new("cargo")
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| format!("error while building '{package_name}'"))?;

    command.wait()?;

    let mode = if release { "release" } else { "debug" };
    let path = format!("target/wasm32-unknown-unknown/{mode}/{package_name}.wasm");
    let to = out.join(format!("{package_name}.wasm"));
    fs::rename(&path, &to)
        .with_context(|| format!("failed to move {} to {}", path, to.display()))?;

    Ok(())
}
