use crate::utils::*;
use anyhow::{ensure, Result};
use std::io;
use std::process::{Command, Stdio};
use std::time::Instant;
use structopt::StructOpt;

pub const MSRV: &'static str = "1.48";

#[derive(StructOpt)]
pub enum Subcommand {
    /// Only runs the fast things (this is used if no command is specified)
    Default,
    /// Runs everything
    Ci,
    /// Checks Rust and Python code formatting with `rustfmt` and `black`
    Fmt,
    /// Runs `clippy`, denying all warnings.
    Clippy,
    /// Runs `cargo llvm-cov` for the PyO3 codebase.
    Coverage(CoverageOpts),
    /// Attempts to render the documentation.
    Doc(DocOpts),
    /// Runs various variations on `cargo test`
    Test,
    /// Runs the tests in examples/ and pytests/
    TestPy,
}

impl Default for Subcommand {
    fn default() -> Self {
        Self::Default
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
        print_metadata()?;

        let start = Instant::now();

        match self {
            Subcommand::Default => {
                crate::fmt::rust::run()?;
                crate::clippy::run()?;
                crate::test::run()?;
                crate::doc::run(DocOpts::default())?;
            }
            Subcommand::Ci => {
                let installed = Installed::new()?;
                crate::fmt::rust::run()?;
                if installed.black {
                    crate::fmt::python::run()?;
                } else {
                    Installed::warn_black()
                };
                crate::clippy::run()?;
                crate::test::run()?;
                crate::doc::run(DocOpts::default())?;
                if installed.nox {
                    crate::pytests::run(None)?;
                } else {
                    Installed::warn_nox()
                };
                crate::llvm_cov::run(CoverageOpts::default())?;
                installed.assert()?
            }

            Subcommand::Doc(opts) => crate::doc::run(opts)?,
            Subcommand::Fmt => {
                crate::fmt::rust::run()?;
                crate::fmt::python::run()?;
            }
            Subcommand::Clippy => crate::clippy::run()?,
            Subcommand::Coverage(opts) => crate::llvm_cov::run(opts)?,
            Subcommand::TestPy => crate::pytests::run(None)?,
            Subcommand::Test => crate::test::run()?,
        };

        let dt = start.elapsed();
        println!("\nFinished program in {} s.", dt.as_secs());

        Ok(())
    }
}

pub fn run(command: &mut Command) -> Result<()> {
    println!("Running: {}", format_command(command));

    let output = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?
        .wait_with_output()?;

    // io::stdout().write_all(&output.stdout).unwrap();
    //  io::stdout().write_all(&output.stderr).unwrap();

    ensure! {
        output.status.success(),
        "process did not run successfully ({exit}): {command}/n {out} {err}",
        exit = match output.status.code() {
            Some(code) => format!("exit code {}", code),
            None => "terminated by signal".into(),
        },
        command = format_command(command),
        out = String::from_utf8_lossy(&output.stdout),
        err = String::from_utf8_lossy(&output.stderr)

    };
    Ok(())
}

#[derive(Copy, Clone, Debug)]
pub struct Installed {
    pub nox: bool,
    pub black: bool,
}

impl Installed {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            nox: Self::nox()?,
            black: Self::black()?,
        })
    }

    pub fn nox() -> anyhow::Result<bool> {
        let output = std::process::Command::new("nox").arg("--version").output();
        match output {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(other) => Err(other)?,
        }
    }

    pub fn warn_nox() {
        eprintln!("Skipping: formatting Python code, because `nox` was not found");
    }

    pub fn black() -> anyhow::Result<bool> {
        let output = std::process::Command::new("black")
            .arg("--version")
            .output();
        match output {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(other) => Err(other)?,
        }
    }

    pub fn warn_black() {
        eprintln!("Skipping: Python code formatting, because `black` was not found.");
    }

    pub fn assert(&self) -> anyhow::Result<()> {
        if self.nox && self.black {
            Ok(())
        } else {
            let mut err =
                String::from("\n\nxtask was unable to run all tests due to some missing programs:");
            if !self.black {
                err.push_str("\n`black` was not installed. (`pip install black`)");
            }
            if !self.nox {
                err.push_str("\n`nox` was not installed. (`pip install nox`)");
            }

            Err(anyhow::anyhow!(err))
        }
    }
}
