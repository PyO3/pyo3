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
fn get_config_vars() -> Result<HashMap<String, String>, String>  {
    let exec_prefix = pkg_config::Config::get_variable(
        "python-2.7", "exec_prefix").unwrap();

    // assume path to the pkg_config python interpreter is 
    // {exec_prefix}/bin/python - this might not hold for all platforms, but
    // the .pc doesn't give us anything else to go on
    let python_path = format!("{}/bin/python", exec_prefix);

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

fn cfg_line_for_var(key: &str, val: &str) -> Option<String> {
    if SYSCONFIG_VALUES.iter().find(|x| **x == key).is_some() {
        // is a value; suffix the key name with the value
        Some(format!("cargo:rustc-cfg={}=\"{}_{}\"", CFG_KEY, key, val))
    } else if val != "0" {
        // is a flag that isn't zero
        Some(format!("cargo:rustc-cfg={}=\"{}\"", CFG_KEY, key))
    } else {
        // is a flag that is zero
        None
    }
}

fn main() {
    // By default, use pkg_config to locate a python 2.7 to use.
    //
    // TODO - allow the user to specify a python via environment variables
    // (PYTHONHOME?) - necessary for systems with no pkg-config, or for
    // compiling against pythons other than the pkg-config one.

    pkg_config::find_library("python-2.7").unwrap();
    let config_map = get_config_vars().unwrap();
    for (key, val) in &config_map {
        match cfg_line_for_var(key, val) {
            Some(line) => println!("{}", line),
            None => ()
        }
    }
}
