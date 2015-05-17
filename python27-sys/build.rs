extern crate pkg_config;

use std::process::Command;
use std::collections::HashMap;

const CFG_KEY: &'static str = "py_sys_config";

// A list of python interpreter compile-time preprocessor defines that 
// we will pick up and pass to rustc via --cfg=py_sys_config={varname};
// this allows using them conditional cfg attributes in the .rs files, so
//
// #[cfg(py_sys_config="{varname}"]
//
// is the equivalent of #ifdef {varname} name in C.
//
// see Misc/SpecialBuilds.txt in the python source for what these mean.
//
// (hrm, this is sort of re-implementing what distutils does, except 
// by passing command line args instead of referring to a python.h)
static SYSCONFIG_FLAGS: [&'static str; 7] = [
    "Py_USING_UNICODE",
    "Py_UNICODE_WIDE",
    "WITH_THREAD",
    "Py_DEBUG",
    "Py_REF_DEBUG",
    "Py_TRACE_REFS",
    "COUNT_ALLOCS",
];

static SYSCONFIG_VALUES: [&'static str; 1] = [
    // cfg doesn't support flags with values, just bools - so flags 
    // below are translated into bools as {varname}_{val} 
    //
    // for example, Py_UNICODE_SIZE_2 or Py_UNICODE_SIZE_4
    "Py_UNICODE_SIZE"
];

/// Examine python's compile flags to pass to cfg by launching
/// the interpreter and printing variables of interest from 
/// sysconfig.get_config_vars.
fn get_config_vars(python_path: &String) -> Result<HashMap<String, String>, String>  {
    let mut script = "import sysconfig; \
config = sysconfig.get_config_vars();".to_owned();

    for k in SYSCONFIG_FLAGS.iter().chain(SYSCONFIG_VALUES.iter()) {
        script.push_str(&format!("print(config.get('{}', 0))", k));
        script.push_str(";");
    }

    let mut cmd = Command::new(python_path);
    cmd.arg("-c").arg(script);

    let out = try!(cmd.output().map_err(|e| {
        format!("failed to run python interpreter `{:?}`: {}", cmd, e)
    }));

    if !out.status.success() {
        let stderr = String::from_utf8(out.stderr).unwrap();
        let mut msg = format!("python script failed with stderr:\n\n");
        msg.push_str(&stderr);
        return Err(msg);
    }

    let stdout = String::from_utf8(out.stdout).unwrap();

    let var_map : HashMap<String, String> = 
        SYSCONFIG_FLAGS.iter().chain(SYSCONFIG_VALUES.iter()).zip(stdout.split('\n'))
            .map(|(&k, v)| (k.to_owned(), v.to_owned()))
            .collect();

    if var_map.len() != SYSCONFIG_VALUES.len() + SYSCONFIG_FLAGS.len() {
        return Err(
            "python stdout len didn't return expected number of lines".to_string());
    }

    return Ok(var_map);
}

fn is_value(key: &str) -> bool {
    SYSCONFIG_VALUES.iter().find(|x| **x == key).is_some()
}

fn cfg_line_for_var(key: &str, val: &str) -> Option<String> {
    if is_value(key) {
        // is a value; suffix the key name with the value
        Some(format!("cargo:rustc-cfg={}=\"{}_{}\"\n", CFG_KEY, key, val))
    } else if val != "0" {
        // is a flag that isn't zero
        Some(format!("cargo:rustc-cfg={}=\"{}\"", CFG_KEY, key))
    } else {
        // is a flag that is zero
        None
    }
}

/// Run a python script using the 'python' located by PATH.
fn run_python_script(script: &str) -> Result<String, String> {
    let mut cmd = Command::new("python");
    cmd.arg("-c").arg(script);

    let out = try!(cmd.output().map_err(|e| {
        format!("failed to run python interpreter `{:?}`: {}", cmd, e)
    }));

    if !out.status.success() {
        let stderr = String::from_utf8(out.stderr).unwrap();
        let mut msg = format!("python script failed with stderr:\n\n");
        msg.push_str(&stderr);
        return Err(msg);
    }

    let out = String::from_utf8(out.stdout).unwrap();
    return Ok(out);
}

#[cfg(not(target_os="macos"))]
fn get_rustc_link_lib(enable_shared: bool) -> Result<String, String> {
    if enable_shared {
        Ok("cargo:rustc-link-lib=python2.7".to_owned())
    } else {
        Ok("cargo:rustc-link-lib=static=python2.7".to_owned())
    }
}

#[cfg(target_os="macos")]
fn get_macos_linkmodel() -> Result<String, String> {
    let script = "import MacOS; print MacOS.linkmodel;";
    let out = run_python_script(script).unwrap();
    Ok(out.trim_right().to_owned())
}

#[cfg(target_os="macos")]
fn get_rustc_link_lib(_: bool) -> Result<String, String> {
    // os x can be linked to a framework or static or dynamic, and 
    // Py_ENABLE_SHARED is wrong; framework means shared library
    match get_macos_linkmodel().unwrap().as_ref() {
        "static" => Ok("cargo:rustc-link-lib=static=python2.7".to_owned()),
        "dynamic" => Ok("cargo:rustc-link-lib=python2.7".to_owned()),
        "framework" => Ok("cargo:rustc-link-lib=python2.7".to_owned()),
        other => Err(format!("unknown linkmodel {}", other))
    }
}

/// Deduce configuration from the 'python' in the current PATH and print
/// cargo vars to stdout.
///
/// Note that if that python isn't version 2.7, this will error.
fn configure_from_path() -> Result<String, String> {
    let script = "import sys; import sysconfig; print(sys.version_info[0:2]); \
print(sysconfig.get_config_var('LIBDIR')); \
print(sysconfig.get_config_var('Py_ENABLE_SHARED')); \
print(sys.exec_prefix);";
    let out = run_python_script(script).unwrap();
    let lines: Vec<&str> = out.split("\n").collect();
    let version: &str = lines[0];
    let libpath: &str = lines[1];
    let enable_shared: &str = lines[2];

    let exec_prefix: &str = lines[3];

    if version != "(2, 7)" {
        return Err(format!("'python' is not version 2.7 (is {})", version));
    }

    println!("{}", get_rustc_link_lib(enable_shared == "1").unwrap());
    println!("cargo:rustc-link-search=native={}", libpath);
    return Ok(format!("{}/bin/python", exec_prefix));
}

/// Deduce configuration from the python-2.7 in pkg-config and print
/// cargo vars to stdout.
fn configure_from_pkgconfig() -> Result<String, String> {
    // this emits relevant build info to stdout, which is picked up by the
    // build chain (funny name for something with side-effects!)
    try!(pkg_config::find_library("python-2.7"));

    let exec_prefix = pkg_config::Config::get_variable("python-2.7", 
        "exec_prefix").unwrap();
    // assume path to the pkg_config python interpreter is 
    // {exec_prefix}/bin/python - this might not hold for all platforms, but
    // the .pc doesn't give us anything else to go on
    return Ok(format!("{}/bin/python", exec_prefix));
}

fn main() {
    // 1. Setup cfg variables so we can do conditional compilation in this 
    // library based on the python interpeter's compilation flags. This is 
    // necessary for e.g. matching the right unicode and threading interfaces.
    //
    // By default, try to use pkgconfig - this seems to be a rust norm.
    //
    // If you want to use a different python, setting the 
    // PYTHON_2.7_NO_PKG_CONFIG environment variable will cause the script 
    // to pick up the python in your PATH (this will work smoothly with an 
    // activated virtualenv). If you have troubles with your shell accepting
    // '.' in a var name, try using 'env'.
    let python_interpreter_path = match configure_from_pkgconfig() {
        Ok(p) => p,
        // no pkgconfig - either it failed or user set the environment 
        // variable "PYTHON_2.7_NO_PKG_CONFIG".
        Err(_) => configure_from_path().unwrap()
    };

    let config_map = get_config_vars(&python_interpreter_path).unwrap();
    for (key, val) in &config_map {
        match cfg_line_for_var(key, val) {
            Some(line) => println!("{}", line),
            None => ()
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
    });
    println!("cargo:python_flags={}", &flags[..flags.len()-1]);
}
