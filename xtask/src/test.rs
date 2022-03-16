use crate::cli::{self, MSRV};
use std::process::Command;

pub fn run() -> anyhow::Result<()> {
    cli::run(
        Command::new("cargo")
            .arg("test")
            .arg("--lib")
            .arg("--no-default-features")
            .arg("--tests")
            .arg("--quiet"),
    )?;

    cli::run(
        Command::new("cargo")
            .arg("test")
            .arg("--no-default-features")
            .arg("--features=full")
            .arg("--quiet"),
    )?;

    cli::run(
        Command::new("cargo")
            .arg("test")
            .arg("--no-default-features")
            .arg("--features=abi3,full")
            .arg("--quiet"),
    )?;

    // If the MSRV toolchain is not installed, this will install it
    cli::run(
        Command::new("rustup")
            .arg("toolchain")
            .arg("install")
            .arg(MSRV),
    )?;

    // Test MSRV
    cli::run(
        Command::new("cargo")
            .arg(format!("+{}", MSRV))
            .arg("test")
            .arg("--no-default-features")
            .arg("--features=full,auto-initialize")
            .arg("--quiet"),
    )?;

    cli::run(
        Command::new("cargo")
            .arg("+nightly")
            .arg("test")
            .arg("--no-default-features")
            .arg("--features=full,nightly")
            .arg("--quiet"),
    )?;

    cli::run(
        Command::new("cargo")
            .arg("test")
            .arg("--manifest-path=pyo3-ffi/Cargo.toml")
            .arg("--quiet"),
    )?;

    cli::run(
        Command::new("cargo")
            .arg("test")
            .arg("--no-default-features")
            .arg("--manifest-path=pyo3-build-config/Cargo.toml")
            .arg("--quiet"),
    )?;

    Ok(())
}
