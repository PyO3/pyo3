use crate::cli;
use std::process::Command;

pub fn run() -> anyhow::Result<()> {
    cli::run(
        Command::new("cargo")
            .arg("clippy")
            .arg("--features=full")
            .arg("--all-targets")
            .arg("--workspace")
            .arg("--")
            .arg("-Dwarnings"),
    )?;
    cli::run(
        Command::new("cargo")
            .arg("clippy")
            .arg("--all-targets")
            .arg("--workspace")
            .arg("--features=abi3,full")
            .arg("--")
            .arg("-Dwarnings"),
    )?;

    Ok(())
}
