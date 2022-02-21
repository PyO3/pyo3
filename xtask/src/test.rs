use crate::cli;
use std::process::Command;

pub fn run() -> anyhow::Result<()> {
    cli::run(
        Command::new("cargo")
            .arg("test")
            .arg("--no-default-features")
            .arg("--lib")
            .arg("--tests"),
    )?;

    cli::run(
        Command::new("cargo")
            .arg("test")
            .arg("--no-default-features")
            .arg("--features=full"),
    )?;

    cli::run(
        Command::new("cargo")
            .arg("+nightly")
            .arg("test")
            .arg("--no-default-features")
            .arg("--features=full,nightly"),
    )?;

    cli::run(
        Command::new("cargo")
            .arg("test")
            .arg("--manifest-path=pyo3-ffi/Cargo.toml"),
    )?;

    Ok(())
}
