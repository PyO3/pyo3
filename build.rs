extern crate pyo3_build_utils;

use pyo3_build_utils::rustc_version::check_rustc_version;
use pyo3_build_utils::py_interpreter::{find_interpreter, emit_cargo_vars_from_configuration, get_config_vars};
use pyo3_build_utils::py_interpreter::PythonVersion;
use pyo3_build_utils::py_interpreter::InterpreterConfig;
use pyo3_build_utils::py_interpreter::version_from_env;
use pyo3_build_utils::py_interpreter::cfg_line_for_var;
use pyo3_build_utils::py_interpreter::is_value;

fn main() {
    check_rustc_version();
    // 1. Setup cfg variables so we can do conditional compilation in this
    // library based on the python interpeter's compilation flags. This is
    // necessary for e.g. matching the right unicode and threading interfaces.
    //
    // This locates the python interpreter based on the PATH, which should
    // work smoothly with an activated virtualenv.
    //
    // If you have troubles with your shell accepting '.' in a var name,
    // try using 'env' (sorry but this isn't our fault - it just has to
    // match the pkg-config package name, which is going to have a . in it).
    let version = match version_from_env() {
        Ok(v) => v,
        Err(_) => PythonVersion {
            major: 3,
            minor: None,
        },
    };

    let interpreter_configuration: InterpreterConfig = find_interpreter(&version).unwrap();

    let flags = emit_cargo_vars_from_configuration(&interpreter_configuration).unwrap();

    let mut config_map = get_config_vars(&interpreter_configuration.path).unwrap();

    config_map.insert("WITH_THREAD".to_owned(), "1".to_owned());

    // WITH_THREAD is always on for 3.7
    if interpreter_configuration.version.major == 3
        && interpreter_configuration.version.minor.unwrap_or(0) >= 7
    {
        config_map.insert("WITH_THREAD".to_owned(), "1".to_owned());
    }

    for (key, val) in &config_map {
        match cfg_line_for_var(key, val) {
            Some(line) => println!("{}", line),
            None => (),
        }
    }

    // 2. Export python interpreter compilation flags as cargo variables that
    // will be visible to dependents. All flags will be available to dependent
    // build scripts in the environment variable DEP_PYTHON27_PYTHON_FLAGS as
    // comma separated list; each item in the list looks like
    //
    // {VAL,FLAG}_{flag_name}=val;
    //
    // FLAG indicates the variable is always 0 or 1
    // VAL indicates it can take on any value
    //
    // rust-cypthon/build.rs contains an example of how to unpack this data
    // into cfg flags that replicate theones present in this library, so
    // you can use the same cfg syntax.
    //let mut flags = flags;
    let flags: String = config_map.iter().fold("".to_owned(), |memo, (key, val)| {
        if is_value(key) {
            memo + format!("VAL_{}={},", key, val).as_ref()
        } else if val != "0" {
            memo + format!("FLAG_{}={},", key, val).as_ref()
        } else {
            memo
        }
    }) + flags.as_str();

    println!(
        "cargo:python_flags={}",
        if flags.len() > 0 {
            &flags[..flags.len() - 1]
        } else {
            ""
        }
    );
}