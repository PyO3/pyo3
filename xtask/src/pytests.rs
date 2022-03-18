use crate::cli;
use anyhow::Result;
use std::{path::Path, process::Command};

pub fn run<'a>(env: impl IntoIterator<Item = (&'a String, &'a String)> + Copy) -> Result<()> {
    cli::run(
        Command::new("nox")
            .arg("--non-interactive")
            .arg("-f")
            .arg(Path::new("pytests").join("noxfile.py"))
            .envs(env),
    )?;

    for entry in std::fs::read_dir("examples")? {
        let path = entry?.path();
        if path.is_dir() && path.join("noxfile.py").exists() {
            cli::run(
                Command::new("nox")
                    .arg("--non-interactive")
                    .arg("-f")
                    .arg(path.join("noxfile.py"))
                    .envs(env),
            )?;
        }
    }
    Ok(())
}
