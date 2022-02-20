use anyhow::{ensure, Result};
use std::{process::Command};
use structopt::StructOpt;
use crate::utils::*;

#[derive(StructOpt)]
pub enum Subcommand {
    /// Runs `cargo llvm-cov` for the PyO3 codebase.
    Coverage(CoverageOpts),
    /// Runs tests in examples/ and pytests/
    TestPy,
}

#[derive(StructOpt)]
pub struct CoverageOpts {
    /// Creates an lcov output instead of printing to the terminal.
    #[structopt(long)]
    pub output_lcov: Option<String>,
}

impl Subcommand {
    pub fn execute(self) -> Result<()> {
        match self {
            Subcommand::Coverage(opts) => crate::llvm_cov::run(opts),
            Subcommand::TestPy => crate::pytests::run(None),
        }
    }
}

pub fn run(command: &mut Command) -> Result<()> {
    println!("running: {}", format_command(command));
    let status = command.spawn()?.wait()?;
    ensure! {
        status.success(),
        "process did not run successfully ({exit}): {command}",
        exit = match status.code() {
            Some(code) => format!("exit code {}", code),
            None => "terminated by signal".into(),
        },
        command = format_command(command),
    };
    Ok(())
}
