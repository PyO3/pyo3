extern crate pyo3_build_utils;

use pyo3_build_utils::{
    py_interpreter::{cfg_line_for_var, find_interpreter, is_value, InterpreterConfig},
    python_version::PythonVersion,
    rustc_version::check_rustc_version,
};
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

fn main() {
    check_rustc_version();
    // 1. Setup cfg variables so we can do conditional compilation in this library based on the
    // python interpeter's compilation flags. This is necessary for e.g. matching the right unicode
    // and threading interfaces.  First check if we're cross compiling, if so, we cannot run the
    // target Python interpreter and have to parse pyconfig.h instead. If we're not cross
    // compiling, locate the python interpreter based on the PATH, which should work smoothly with
    // an activated virtualenv, and load from there.
    //
    // If you have troubles with your shell accepting '.' in a var name,
    // try using 'env' (sorry but this isn't our fault - it just has to
    // match the pkg-config package name, which is going to have a . in it).
    let cross_compiling =
        env::var("PYO3_CROSS_INCLUDE_DIR").is_ok() && env::var("PYO3_CROSS_LIB_DIR").is_ok();

    let interpreter_config = if cross_compiling {
        InterpreterConfig::from_cross_compile_info()
            .expect("Error when loading cross compile settings.")
    } else {
        let version = PythonVersion::from_env().unwrap_or_default();
        find_interpreter(&version).expect("Interpreter not found.")
    };

    let config_map = interpreter_config
        .get_config_vars()
        .expect("Failed to load configuration variables");

    let flags = interpreter_config.emit_cargo_vars();

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
    // into cfg flags that replicate the ones present in this library, so
    // you can use the same cfg syntax.
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

    if env::var_os("TARGET") == Some("x86_64-apple-darwin".into()) {
        // TODO: Find out how we can set -undefined dynamic_lookup here (if this is possible)
    }

    let env_vars = ["LD_LIBRARY_PATH", "PATH", "PYTHON_SYS_EXECUTABLE", "LIB"];

    for var in env_vars.iter() {
        println!("cargo:rerun-if-env-changed={}", var);
    }
}
