use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::collections::HashMap;
use regex::Regex;
use std::fs::File;
use std::io::{BufReader, BufRead};

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

/// Attempts to parse the header at the given path, returning a map of definitions to their values.
/// Each entry in the map directly corresponds to a `#define` in the given header.
pub fn parse_header_defines<P: AsRef<Path>>(header_path: P) -> Result<HashMap<String, String>, String> {
    // This regex picks apart a C style, single line `#define` statement into an identifier and a
    // value. e.g. for the line `#define Py_DEBUG 1`, this regex will capture `Py_DEBUG` into
    // `ident` and `1` into `value`.
    let define_regex =
        Regex::new(r"^\s*#define\s+(?P<ident>[a-zA-Z0-9_]+)\s+(?P<value>.+)\s*$").unwrap();

    let header_file = File::open(header_path.as_ref()).map_err(|e| e.to_string())?;
    let header_reader = BufReader::new(&header_file);

    let definitions = header_reader
        .lines()
        .filter_map(|maybe_line| {
            let line = maybe_line.unwrap_or_else(|err| {
                panic!("failed to read {}: {}", header_path.as_ref().display(), err);
            });
            let captures = define_regex.captures(&line)?;

            if captures.name("ident").is_some() && captures.name("value").is_some() {
                Some((
                    captures.name("ident").unwrap().as_str().to_owned(),
                    captures.name("value").unwrap().as_str().to_owned(),
                ))
            } else {
                None
            }
        })
        .collect();

    Ok(definitions)
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
