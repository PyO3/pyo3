use crate::cli;
use crate::cli::CoverageOpts;
use crate::utils::*;
use anyhow::{Context, Result};
use std::{collections::HashMap, process::Command};

/// Runs `cargo llvm-cov` for the PyO3 codebase.
pub fn run(opts: CoverageOpts) -> Result<()> {
    let env = get_coverage_env()?;

    cli::run(
        Command::new("cargo")
            .args(&["llvm-cov", "clean", "--workspace"])
            .envs(&env),
    )?;

    cli::run(
        Command::new("cargo")
            .args(&["test", "--manifest-path", "pyo3-build-config/Cargo.toml"])
            .envs(&env),
    )?;
    cli::run(
        Command::new("cargo")
            .args(&["test", "--manifest-path", "pyo3-macros-backend/Cargo.toml"])
            .envs(&env),
    )?;
    cli::run(
        Command::new("cargo")
            .args(&["test", "--manifest-path", "pyo3-macros/Cargo.toml"])
            .envs(&env),
    )?;

    cli::run(Command::new("cargo").arg("test").envs(&env))?;
    cli::run(
        Command::new("cargo")
            .args(&["test", "--features", "abi3"])
            .envs(&env),
    )?;
    cli::run(
        Command::new("cargo")
            .args(&["test", "--features", "full"])
            .envs(&env),
    )?;
    cli::run(
        Command::new("cargo")
            .args(&["test", "--features", "abi3 full"])
            .envs(&env),
    )?;

    crate::pytests::run(&env)?;

    cli::run(
        llvm_cov_command(&["--no-run", "--lcov", "--output-path", &opts.output_lcov]).envs(&env),
    )?;

    Ok(())
}

fn llvm_cov_command(args: &[&str]) -> Command {
    let mut command = Command::new("cargo");
    command
        .args(&[
            "llvm-cov",
            "--package=pyo3",
            "--package=pyo3-build-config",
            "--package=pyo3-macros-backend",
            "--package=pyo3-macros",
            "--package=pyo3-ffi",
        ])
        .args(args);
    command
}

fn get_coverage_env() -> Result<HashMap<String, String>> {
    let mut env = HashMap::new();

    let output = cli::run_with_output(&mut llvm_cov_command(&["show-env"])).context("Unable to run llvm-cov. If it is not installed, you can install it with `cargo install cargo-llvm-cov`.")?;

    let output = std::str::from_utf8(&output.stdout)?;

    for line in output.trim().split('\n') {
        let (key, value) = split_once(line, '=')
            .context("expected '=' in each line of output from llvm-cov show-env")?;
        env.insert(key.to_owned(), value.trim_matches('"').to_owned());
    }

    // Ensure that examples/ and pytests/ all build to the correct target directory to collect
    // coverage artifacts.
    env.insert(
        "CARGO_TARGET_DIR".to_owned(),
        env.get("CARGO_LLVM_COV_TARGET_DIR").unwrap().to_owned(),
    );

    Ok(env)
}
