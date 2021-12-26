use anyhow::{Context, Result};
use std::{collections::HashMap, process::Command};
use structopt::StructOpt;

#[derive(StructOpt)]
enum Subcommand {
    /// Runs `cargo llvm-cov` for the PyO3 codebase.
    Coverage,
    /// Runs tests in examples/ and pytests/
    TestPy,
}

impl Subcommand {
    fn execute(self) -> Result<()> {
        match self {
            Subcommand::Coverage => subcommand_coverage(),
            Subcommand::TestPy => run_python_tests(None),
        }
    }
}

fn main() -> Result<()> {
    Subcommand::from_args().execute()
}

/// Runs `cargo llvm-cov` for the PyO3 codebase.
fn subcommand_coverage() -> Result<()> {
    run(&mut llvm_cov_command(&["clean", "--workspace"]))?;
    run(&mut llvm_cov_command(&["--no-report"]))?;

    // FIXME: add various feature combinations using 'full' feature.
    // run(&mut llvm_cov_command(&["--no-report"]))?;

    // XXX: the following block doesn't work until https://github.com/taiki-e/cargo-llvm-cov/pull/115 is merged
    let env = get_coverage_env()?;
    run_python_tests(&env)?;
    // (after here works with stable llvm-cov)

    // TODO: add an argument to make it possible to generate lcov report & use this in CI.
    run(&mut llvm_cov_command(&["--no-run", "--summary-only"]))?;
    Ok(())
}

fn run(command: &mut Command) -> Result<()> {
    println!("running: {}", format_command(command));
    command.spawn()?.wait()?;
    Ok(())
}

fn llvm_cov_command(args: &[&str]) -> Command {
    let mut command = Command::new("cargo");
    command.args(&["llvm-cov", "--package=pyo3"]).args(args);
    command
}

fn run_python_tests<'a>(
    env: impl IntoIterator<Item = (&'a String, &'a String)> + Copy,
) -> Result<()> {
    for entry in std::fs::read_dir("pytests")? {
        let path = entry?.path();
        if path.is_dir() && path.join("tox.ini").exists() {
            run(Command::new("tox").arg("-c").arg(path).envs(env))?;
        }
    }
    for entry in std::fs::read_dir("examples")? {
        let path = entry?.path();
        if path.is_dir() && path.join("tox.ini").exists() {
            run(Command::new("tox").arg("-c").arg(path).envs(env))?;
        }
    }
    Ok(())
}

fn get_coverage_env() -> Result<HashMap<String, String>> {
    let mut env = HashMap::new();

    let output = String::from_utf8(llvm_cov_command(&["show-env"]).output()?.stdout)?;

    for line in output.trim().split('\n') {
        let (key, value) = split_once(line, '=').context("expected '=' in each output line")?;
        env.insert(key.to_owned(), value.trim_matches('"').to_owned());
    }

    env.insert("TOX_TESTENV_PASSENV".to_owned(), "*".to_owned());
    env.insert("RUSTUP_TOOLCHAIN".to_owned(), "nightly".to_owned());

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
