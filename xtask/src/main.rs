use anyhow::{ensure, Context, Result};
use std::{collections::HashMap, path::Path, process::Command};
use structopt::StructOpt;

#[derive(StructOpt)]
enum Subcommand {
    /// Runs `cargo llvm-cov` for the PyO3 codebase.
    Coverage(CoverageOpts),
    /// Runs tests in examples/ and pytests/
    TestPy,
}

#[derive(StructOpt)]
struct CoverageOpts {
    /// Creates an lcov output instead of printing to the terminal.
    #[structopt(long)]
    output_lcov: Option<String>,
}

impl Subcommand {
    fn execute(self) -> Result<()> {
        match self {
            Subcommand::Coverage(opts) => subcommand_coverage(opts),
            Subcommand::TestPy => run_python_tests(None),
        }
    }
}

fn main() -> Result<()> {
    Subcommand::from_args().execute()
}

/// Runs `cargo llvm-cov` for the PyO3 codebase.
fn subcommand_coverage(opts: CoverageOpts) -> Result<()> {
    let env = get_coverage_env()?;

    run(llvm_cov_command(&["clean", "--workspace"]).envs(&env))?;

    run(Command::new("cargo")
        .args(&["test", "--manifest-path", "pyo3-build-config/Cargo.toml"])
        .envs(&env))?;
    run(Command::new("cargo")
        .args(&["test", "--manifest-path", "pyo3-macros-backend/Cargo.toml"])
        .envs(&env))?;
    run(Command::new("cargo")
        .args(&["test", "--manifest-path", "pyo3-macros/Cargo.toml"])
        .envs(&env))?;

    run(Command::new("cargo").arg("test").envs(&env))?;
    run(Command::new("cargo")
        .args(&["test", "--features", "abi3"])
        .envs(&env))?;
    run(Command::new("cargo")
        .args(&["test", "--features", "full"])
        .envs(&env))?;
    run(Command::new("cargo")
        .args(&["test", "--features", "abi3 full"])
        .envs(&env))?;

    run_python_tests(&env)?;

    match opts.output_lcov {
        Some(path) => {
            run(llvm_cov_command(&["--no-run", "--lcov", "--output-path", &path]).envs(&env))?
        }
        None => run(llvm_cov_command(&["--no-run", "--summary-only"]).envs(&env))?,
    }

    Ok(())
}

fn run(command: &mut Command) -> Result<()> {
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

fn get_output(command: &mut Command) -> Result<std::process::Output> {
    let output = command.output()?;
    ensure! {
        output.status.success(),
        "process did not run successfully ({exit}): {command}",
        exit = match output.status.code() {
            Some(code) => format!("exit code {}", code),
            None => "terminated by signal".into(),
        },
        command = format_command(command),
    };
    Ok(output)
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

fn run_python_tests<'a>(
    env: impl IntoIterator<Item = (&'a String, &'a String)> + Copy,
) -> Result<()> {
    run(Command::new("nox")
        .arg("--non-interactive")
        .arg("-f")
        .arg(Path::new("pytests").join("noxfile.py"))
        .envs(env))?;

    for entry in std::fs::read_dir("examples")? {
        let path = entry?.path();
        if path.is_dir() && path.join("noxfile.py").exists() {
            run(Command::new("nox")
                .arg("--non-interactive")
                .arg("-f")
                .arg(path.join("noxfile.py"))
                .envs(env))?;
        }
    }
    Ok(())
}

fn get_coverage_env() -> Result<HashMap<String, String>> {
    let mut env = HashMap::new();

    let output = String::from_utf8(llvm_cov_command(&["show-env"]).output()?.stdout)?;

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

    // Coverage only works on nightly.
    let rustc_version =
        String::from_utf8(get_output(Command::new("rustc").arg("--version"))?.stdout)
            .context("failed to parse rust version as utf8")?;
    if !rustc_version.contains("nightly") {
        env.insert("RUSTUP_TOOLCHAIN".to_owned(), "nightly".to_owned());
    }

    Ok(env)
}

// Replacement for str.split_once() on Rust older than 1.52
#[rustversion::before(1.52)]
fn split_once(s: &str, pat: char) -> Option<(&str, &str)> {
    let mut iter = s.splitn(2, pat);
    Some((iter.next()?, iter.next()?))
}

#[rustversion::since(1.52)]
fn split_once(s: &str, pat: char) -> Option<(&str, &str)> {
    s.split_once(pat)
}

#[rustversion::since(1.57)]
fn format_command(command: &Command) -> String {
    let mut buf = String::new();
    buf.push('`');
    buf.push_str(&command.get_program().to_string_lossy());
    for arg in command.get_args() {
        buf.push(' ');
        buf.push_str(&arg.to_string_lossy());
    }
    buf.push('`');
    buf
}

#[rustversion::before(1.57)]
fn format_command(command: &Command) -> String {
    // Debug impl isn't as nice as the above, but will do on < 1.57
    format!("{:?}", command)
}
