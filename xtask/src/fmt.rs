use crate::cli;
use std::process::Command;

pub fn run() -> anyhow::Result<()> {
    cli::run(
        Command::new("cargo")
            .arg("fmt")
            .arg("--all")
            .arg("--")
            .arg("--check"),
    )?;

    cli::run(Command::new("black").arg(".").arg("--check"))?;

    Ok(())
}
