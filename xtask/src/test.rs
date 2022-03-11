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

    // Install 1.48 for testing msrv
    cli::run(
        Command::new("rustup")
            .arg("toolchain")
            .arg("install")
            .arg("1.48"),
    )?;

    // Test msrv
    cli::run(
        Command::new("cargo")
            .arg("+1.48")
            .arg("test")
            .arg("--no-default-features")
            .arg("--features=full,auto-initialize"),
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
