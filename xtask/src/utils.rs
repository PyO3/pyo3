use anyhow::ensure;
use std::process::Command;

pub fn get_output(command: &mut Command) -> anyhow::Result<std::process::Output> {
    let output = command.output()?;
    ensure! {
        output.status.success(),
        "process did not run successfully ({exit}): {command:?}",
        exit = match output.status.code() {
            Some(code) => format!("exit code {}", code),
            None => "terminated by signal".into(),
        },
        command = command,
    };
    Ok(output)
}

pub fn print_metadata() -> anyhow::Result<()> {
    let rustc_output = std::process::Command::new("rustc")
        .arg("--version")
        .arg("--verbose")
        .output()?;
    let rustc_version = core::str::from_utf8(&rustc_output.stdout).unwrap();
    println!("Metadata: \n\n{}", rustc_version);

    let py_output = std::process::Command::new("python")
        .arg("--version")
        .arg("-V")
        .output()?;
    let py_version = core::str::from_utf8(&py_output.stdout).unwrap();
    println!("{}", py_version);

    Ok(())
}
