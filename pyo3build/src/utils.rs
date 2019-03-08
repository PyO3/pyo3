use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn canonicalize_executable<P>(exe_name: P) -> Option<PathBuf>
where
    P: AsRef<Path>,
{
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .filter_map(|dir| {
                let full_path = dir.join(&exe_name);
                if full_path.is_file() {
                    Some(full_path)
                } else {
                    None
                }
            })
            .next()
    })
}

/// Run a python script using the specified interpreter binary.
/// Returns an error if python printer anything to stderr.
pub fn run_python_script(
    interpreter_path: impl AsRef<Path>,
    script: &str,
) -> Result<String, String> {
    let mut cmd = Command::new(interpreter_path.as_ref());
    cmd.arg("-c").arg(script);

    let out = cmd
        .output()
        .map_err(|e| format!("failed to run python interpreter `{:?}`: {}", cmd, e))?;

    if !out.status.success() {
        let stderr = String::from_utf8(out.stderr).unwrap();
        let mut msg = format!("python script failed with stderr:\n\n");
        msg.push_str(&stderr);
        return Err(msg);
    }

    let out = String::from_utf8(out.stdout).unwrap();
    return Ok(out);
}
