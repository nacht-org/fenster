use anyhow::Context;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Deserialize, Debug)]
struct Cargo {
    workspace: Workspace,
}

#[derive(Deserialize, Debug)]
struct Workspace {
    members: Vec<String>,
}

pub fn build(out: PathBuf) -> anyhow::Result<()> {
    let members = {
        let content = fs::read_to_string("Cargo.toml").context("unable to open 'Cargo.toml'")?;
        let cargo = toml::from_str::<Cargo>(&content)
            .context("failed to parse workspace members from 'Cargo.toml'")?;
        cargo.workspace.members
    };

    let extensions = members.iter().filter(|v| v.starts_with("extensions/"));

    for extension in extensions {
        let package = extension.replace("s/", "_");

        let mut command = Command::new("cargo")
            .args([
                "build",
                "-p",
                &package,
                "--release",
                "--target",
                "wasm32-unknown-unknown",
            ])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| format!("error while building '{package}'"))?;

        command.wait()?;

        let path = format!("target/wasm32-unknown-unknown/release/{package}.wasm");
        let to = out.join(format!("{package}.wasm"));
        fs::rename(&path, &to)
            .with_context(|| format!("failed to move {} to {}", path, to.display()))?;
    }

    Ok(())
}
