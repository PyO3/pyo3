use anyhow::ensure;
use std::process::Command;

// Replacement for str.split_once() on Rust older than 1.52
#[rustversion::before(1.52)]
pub fn split_once(s: &str, pat: char) -> Option<(&str, &str)> {
    let mut iter = s.splitn(2, pat);
    Some((iter.next()?, iter.next()?))
}

#[rustversion::since(1.52)]
pub fn split_once(s: &str, pat: char) -> Option<(&str, &str)> {
    s.split_once(pat)
}

#[rustversion::since(1.57)]
pub fn format_command(command: &Command) -> String {
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
pub fn format_command(command: &Command) -> String {
    // Debug impl isn't as nice as the above, but will do on < 1.57
    format!("{:?}", command)
}

pub fn get_output(command: &mut Command) -> anyhow::Result<std::process::Output> {
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
