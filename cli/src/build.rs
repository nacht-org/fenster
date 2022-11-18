use anyhow::Context;
use serde::Deserialize;
use std::fs;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

#[derive(Deserialize, Debug)]
struct Cargo {
    workspace: Workspace,
}

#[derive(Deserialize, Debug)]
struct Workspace {
    members: Vec<String>,
}

pub fn build() -> anyhow::Result<()> {
    let members = {
        let content = fs::read_to_string("Cargo.toml").context("unable to open 'Cargo.toml'")?;
        let cargo = toml::from_str::<Cargo>(&content)
            .context("failed to parse workspace members from 'Cargo.toml'")?;
        cargo.workspace.members
    };

    println!("{members:?}");

    let extensions = members.iter().filter(|v| v.starts_with("extensions/"));
    println!("{:?}", extensions.clone().collect::<Vec<_>>());

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
            .stdout(Stdio::piped())
            .spawn()
            .with_context(|| format!("error while building '{package}'"))?;

        {
            let stdout = command.stdout.as_mut().unwrap();
            let stdout_reader = BufReader::new(stdout);
            let stdout_lines = stdout_reader.lines();

            for line in stdout_lines {
                println!("{}", line?);
            }
        }

        command.wait()?;

        let path = format!("target/wasm32-unknown-unknown/release/{package}.wasm");
        let to = format!("dist/{package}.wasm");
        fs::rename(&path, &to).with_context(|| format!("failed to move {path} to {to}"))?;
    }

    Ok(())
}
