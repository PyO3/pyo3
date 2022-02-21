use crate::utils::*;
use anyhow::{ensure, Result};
use std::process::Command;
use structopt::StructOpt;

#[derive(StructOpt)]
pub enum Subcommand {
    /// Runs everything
    All,
    /// Checks Rust and Python code formatting with `rustfmt` and `black`
    Fmt,
    /// Runs clippy, denying all warnings.
    Clippy,
    /// Runs `cargo llvm-cov` for the PyO3 codebase.
    Coverage(CoverageOpts),
    /// Render documentation
    Doc(DocOpts),
    /// Runs various incantations of `cargo test`
    Test,
    /// Runs tests in examples/ and pytests/
    TestPy,
}

impl Default for Subcommand {
    fn default() -> Self {
        Self::All
    }
}

#[derive(StructOpt, Default)]
pub struct CoverageOpts {
    /// Creates an lcov output instead of printing to the terminal.
    #[structopt(long)]
    pub output_lcov: Option<String>,
}

#[derive(StructOpt)]
pub struct DocOpts {
    /// Whether to run the docs using nightly rustdoc
    #[structopt(long)]
    pub stable: bool,
    /// Whether to open the docs after rendering.
    #[structopt(long)]
    pub open: bool,
    /// Whether to show the private and hidden API.
    #[structopt(long)]
    pub internal: bool,
}

impl Default for DocOpts {
    fn default() -> Self {
        Self {
            stable: true,
            open: false,
            internal: false,
        }
    }
}

impl Subcommand {
    pub fn execute(self) -> Result<()> {
        match self {
            Subcommand::All => {
                crate::fmt::run()?;
                crate::clippy::run()?;
                crate::test::run()?;
                crate::doc::run(DocOpts::default())?;
                crate::pytests::run(None)?;
                crate::llvm_cov::run(CoverageOpts::default())?;
            }

            Subcommand::Doc(opts) => crate::doc::run(opts)?,
            Subcommand::Fmt => crate::fmt::run()?,
            Subcommand::Clippy => crate::clippy::run()?,
            Subcommand::Coverage(opts) => crate::llvm_cov::run(opts)?,
            Subcommand::TestPy => crate::pytests::run(None)?,
            Subcommand::Test => crate::test::run()?,
        };

        Ok(())
    }
}

pub fn run(command: &mut Command) -> Result<()> {
    println!("Running: {}", format_command(command));
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
