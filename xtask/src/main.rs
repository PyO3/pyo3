use anyhow::Result;
use clap::Parser;
use std::{collections::HashMap, process::Command};

#[derive(Parser)]
enum Subcommand {
    Coverage,
}

fn main() -> Result<()> {
    match Subcommand::parse() {
        Subcommand::Coverage => {
            run(&mut llvm_cov_command(&["clean", "--workspace"]))?;
            // FIXME: run (including with various feature combinations)
            // run(&mut llvm_cov_command(&["--no-report"]))?;
            let env = get_coverage_env()?;
            for entry in std::fs::read_dir("pytests")? {
                let path = entry?.path();
                if path.is_dir() {
                    run(Command::new("tox").arg("-c").arg(path).envs(&env))?;
                }
            }
            // FIXME: also run for examples
            // FIXME: make it possible to generate lcov report too
            run(&mut llvm_cov_command(&["--no-run", "--summary-only"]))?;
        }
    }
    Ok(())
}

fn run(command: &mut Command) -> Result<()> {
    command.spawn()?.wait()?;
    Ok(())
}

fn llvm_cov_command(args: &[&str]) -> Command {
    let mut command = Command::new("cargo");
    command.args(["llvm-cov", "--package=pyo3"]).args(args);
    command
}

fn get_coverage_env() -> Result<HashMap<String, String>> {
    let mut env = HashMap::new();

    let output = String::from_utf8(llvm_cov_command(&["show-env"]).output()?.stdout)?;

    for line in output.trim().split('\n') {
        // TODO use split_once on MSRV 1.52
        let mut iter = line.splitn(2, '=');
        env.insert(iter.next().unwrap().into(), iter.next().unwrap().trim_matches('"').into());
    }

    env.insert("TOX_TESTENV_PASSENV".to_owned(), "*".to_owned());
    env.insert("RUSTUP_TOOLCHAIN".to_owned(), "nightly".to_owned());

    Ok(env)
}
