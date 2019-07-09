use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::convert::AsRef;
use std::env;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::exit;
use std::process::Command;
use std::process::Stdio;
use version_check::{is_min_date, is_min_version, supports_features};

/// Specifies the minimum nightly version needed to compile pyo3.
/// Keep this synced up with the travis ci config,
/// But note that this is the rustc version which can be lower than the nightly version
const MIN_DATE: &'static str = "2019-06-21";
const MIN_VERSION: &'static str = "1.37.0-nightly";

/// Information returned from python interpreter
#[derive(Deserialize, Debug)]
struct InterpreterConfig {
    version: PythonVersion,
    libdir: Option<String>,
    shared: bool,
    ld_version: String,
    /// Prefix used for determining the directory of libpython
    base_prefix: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum PythonInterpreterKind {
    CPython,
    PyPy,
}

#[derive(Deserialize, Debug, Clone)]
struct PythonVersion {
    major: u8,
    // minor == None means any minor version will do
    minor: Option<u8>,
    implementation: PythonInterpreterKind,
}

impl PartialEq for PythonVersion {
    fn eq(&self, o: &PythonVersion) -> bool {
        self.major == o.major && (self.minor.is_none() || self.minor == o.minor)
    }
}

impl fmt::Display for PythonVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.major.fmt(f)?;
        f.write_str(".")?;
        match self.minor {
            Some(minor) => minor.fmt(f)?,
            None => f.write_str("*")?,
        };
        Ok(())
    }
}

const PY3_MIN_MINOR: u8 = 5;

const CFG_KEY: &'static str = "py_sys_config";

/// A list of python interpreter compile-time preprocessor defines that
/// we will pick up and pass to rustc via --cfg=py_sys_config={varname};
/// this allows using them conditional cfg attributes in the .rs files, so
///
/// #[cfg(py_sys_config="{varname}"]
///
/// is the equivalent of #ifdef {varname} name in C.
///
/// see Misc/SpecialBuilds.txt in the python source for what these mean.
///
/// (hrm, this is sort of re-implementing what distutils does, except
/// by passing command line args instead of referring to a python.h)
#[cfg(not(target_os = "windows"))]
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
    "Py_UNICODE_SIZE", // note - not present on python 3.3+, which is always wide
];

/// Attempts to parse the header at the given path, returning a map of definitions to their values.
/// Each entry in the map directly corresponds to a `#define` in the given header.
fn parse_header_defines<P: AsRef<Path>>(header_path: P) -> Result<HashMap<String, String>, String> {
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

fn fix_config_map(mut config_map: HashMap<String, String>) -> HashMap<String, String> {
    if let Some("1") = config_map.get("Py_DEBUG").as_ref().map(|s| s.as_str()) {
        config_map.insert("Py_REF_DEBUG".to_owned(), "1".to_owned());
        config_map.insert("Py_TRACE_REFS".to_owned(), "1".to_owned());
        config_map.insert("COUNT_ALLOCS".to_owned(), "1".to_owned());
    }

    config_map
}

fn load_cross_compile_info() -> Result<(InterpreterConfig, HashMap<String, String>), String> {
    let python_include_dir = env::var("PYO3_CROSS_INCLUDE_DIR").unwrap();
    let python_include_dir = Path::new(&python_include_dir);

    let patchlevel_defines = parse_header_defines(python_include_dir.join("patchlevel.h"))?;

    let major = patchlevel_defines
        .get("PY_MAJOR_VERSION")
        .and_then(|major| major.parse::<u8>().ok())
        .expect("PY_MAJOR_VERSION undefined");

    let minor = patchlevel_defines
        .get("PY_MINOR_VERSION")
        .and_then(|minor| minor.parse::<u8>().ok())
        .expect("PY_MINOR_VERSION undefined");

    let python_version = PythonVersion {
        major,
        minor: Some(minor),
        implementation: PythonInterpreterKind::CPython,
    };

    let config_map = parse_header_defines(python_include_dir.join("pyconfig.h"))?;
    let shared = match config_map
        .get("Py_ENABLE_SHARED")
        .map(|x| x.as_str())
        .ok_or("Py_ENABLE_SHARED is not defined".to_string())?
    {
        "1" | "true" | "True" => true,
        "0" | "false" | "False" => false,
        _ => panic!("Py_ENABLE_SHARED must be a bool (1/true/True or 0/false/False"),
    };

    let intepreter_config = InterpreterConfig {
        version: python_version,
        libdir: Some(env::var("PYO3_CROSS_LIB_DIR").expect("PYO3_CROSS_LIB_DIR is not set")),
        shared,
        ld_version: "".to_string(),
        base_prefix: "".to_string(),
    };

    Ok((intepreter_config, fix_config_map(config_map)))
}

/// Examine python's compile flags to pass to cfg by launching
/// the interpreter and printing variables of interest from
/// sysconfig.get_config_vars.
#[cfg(not(target_os = "windows"))]
fn get_config_vars(python_path: &str) -> Result<HashMap<String, String>, String> {
    // FIXME: We can do much better here using serde:
    // import json, sysconfig; print(json.dumps({k:str(v) for k, v in sysconfig.get_config_vars().items()}))

    let mut script = "import sysconfig; \
                      config = sysconfig.get_config_vars();"
        .to_owned();

    for k in SYSCONFIG_FLAGS.iter().chain(SYSCONFIG_VALUES.iter()) {
        script.push_str(&format!(
            "print(config.get('{}', {}));",
            k,
            if is_value(k) { "None" } else { "0" }
        ));
    }

    let stdout = run_python_script(python_path, &script)?;
    let split_stdout: Vec<&str> = stdout.trim_end().lines().collect();
    if split_stdout.len() != SYSCONFIG_VALUES.len() + SYSCONFIG_FLAGS.len() {
        return Err(format!(
            "python stdout len didn't return expected number of lines: {}",
            split_stdout.len()
        ));
    }
    let all_vars = SYSCONFIG_FLAGS.iter().chain(SYSCONFIG_VALUES.iter());
    let all_vars = all_vars.zip(split_stdout.iter()).fold(
        HashMap::new(),
        |mut memo: HashMap<String, String>, (&k, &v)| {
            if !(v.to_owned() == "None" && is_value(k)) {
                memo.insert(k.to_owned(), v.to_owned());
            }
            memo
        },
    );

    Ok(fix_config_map(all_vars))
}

#[cfg(target_os = "windows")]
fn get_config_vars(_: &str) -> Result<HashMap<String, String>, String> {
    // sysconfig is missing all the flags on windows, so we can't actually
    // query the interpreter directly for its build flags.
    //
    // For the time being, this is the flags as defined in the python source's
    // PC\pyconfig.h. This won't work correctly if someone has built their
    // python with a modified pyconfig.h - sorry if that is you, you will have
    // to comment/uncomment the lines below.
    let mut map: HashMap<String, String> = HashMap::new();
    map.insert("Py_USING_UNICODE".to_owned(), "1".to_owned());
    map.insert("Py_UNICODE_WIDE".to_owned(), "0".to_owned());
    map.insert("WITH_THREAD".to_owned(), "1".to_owned());
    map.insert("Py_UNICODE_SIZE".to_owned(), "2".to_owned());

    // This is defined #ifdef _DEBUG. The visual studio build seems to produce
    // a specially named pythonXX_d.exe and pythonXX_d.dll when you build the
    // Debug configuration, which this script doesn't currently support anyway.
    // map.insert("Py_DEBUG", "1");

    // Uncomment these manually if your python was built with these and you want
    // the cfg flags to be set in rust.
    //
    // map.insert("Py_REF_DEBUG", "1");
    // map.insert("Py_TRACE_REFS", "1");
    // map.insert("COUNT_ALLOCS", 1");
    Ok(map)
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

/// Run a python script using the specified interpreter binary.
fn run_python_script(interpreter: &str, script: &str) -> Result<String, String> {
    let out = Command::new(interpreter)
        .args(&["-c", script])
        .stderr(Stdio::inherit())
        .output();

    let out = match out {
        Err(err) => {
            if err.kind() == io::ErrorKind::NotFound {
                return Err(format!(
                    "Could not find any interpreter at {}, \
                     are you sure you have python installed on your PATH?",
                    interpreter
                ));
            } else {
                return Err(format!(
                    "Failed to run the python interpreter at {}: {}",
                    interpreter, err
                ));
            }
        }
        Ok(ok) => ok,
    };

    if !out.status.success() {
        return Err(format!("python script failed"));
    }

    Ok(String::from_utf8(out.stdout).unwrap())
}

fn get_library_link_name(version: &PythonVersion, ld_version: &str) -> String {
    if cfg!(target_os = "windows") {
        let minor_or_empty_string = match version.minor {
            Some(minor) => format!("{}", minor),
            None => String::new(),
        };
        match version.implementation {
            PythonInterpreterKind::CPython => {
                format!("python{}{}", version.major, minor_or_empty_string)
            }
            PythonInterpreterKind::PyPy => format!("pypy{}-c", version.major),
        }
    } else {
        match version.implementation {
            PythonInterpreterKind::CPython => format!("python{}", ld_version),
            PythonInterpreterKind::PyPy => format!("pypy{}-c", version.major),
        }
    }
}

#[cfg(not(target_os = "macos"))]
#[cfg(not(target_os = "windows"))]
fn get_rustc_link_lib(
    version: &PythonVersion,
    ld_version: &str,
    enable_shared: bool,
) -> Result<String, String> {
    if enable_shared {
        Ok(format!(
            "cargo:rustc-link-lib={}",
            get_library_link_name(&version, ld_version)
        ))
    } else {
        Ok(format!(
            "cargo:rustc-link-lib=static={}",
            get_library_link_name(&version, ld_version)
        ))
    }
}

#[cfg(target_os = "macos")]
fn get_macos_linkmodel() -> Result<String, String> {
    let script = r#"
import sysconfig

if sysconfig.get_config_var("PYTHONFRAMEWORK"):
    print("framework")
elif sysconfig.get_config_var("Py_ENABLE_SHARED"):
    print("shared")
else:
    print("static")
"#;
    let out = run_python_script("python", script).unwrap();
    Ok(out.trim_end().to_owned())
}

#[cfg(target_os = "macos")]
fn get_rustc_link_lib(
    version: &PythonVersion,
    ld_version: &str,
    _: bool,
) -> Result<String, String> {
    // os x can be linked to a framework or static or dynamic, and
    // Py_ENABLE_SHARED is wrong; framework means shared library
    match get_macos_linkmodel().unwrap().as_ref() {
        "static" => Ok(format!(
            "cargo:rustc-link-lib=static={}",
            get_library_link_name(&version, ld_version)
        )),
        "shared" => Ok(format!(
            "cargo:rustc-link-lib={}",
            get_library_link_name(&version, ld_version)
        )),
        "framework" => Ok(format!(
            "cargo:rustc-link-lib={}",
            get_library_link_name(&version, ld_version)
        )),
        other => Err(format!("unknown linkmodel {}", other)),
    }
}

#[cfg(target_os = "windows")]
fn get_rustc_link_lib(
    version: &PythonVersion,
    ld_version: &str,
    _: bool,
) -> Result<String, String> {
    // Py_ENABLE_SHARED doesn't seem to be present on windows.
    Ok(format!(
        "cargo:rustc-link-lib=pythonXY:{}",
        get_library_link_name(&version, ld_version)
    ))
}

/// Locate a suitable python interpreter and extract config from it.
///
/// The following locations are checked in the order listed:
///
/// 1. If `PYTHON_SYS_EXECUTABLE` is set, this intepreter is used and an error is raised if the
/// version doesn't match.
/// 2. `python`
/// 3. `python{major version}`
/// 4. `python{major version}.{minor version}`
///
/// If none of the above works, an error is returned
fn find_interpreter_and_get_config() -> Result<(InterpreterConfig, HashMap<String, String>), String>
{
    if let Some(sys_executable) = env::var_os("PYTHON_SYS_EXECUTABLE") {
        let interpreter_path = sys_executable
            .to_str()
            .expect("Unable to get PYTHON_SYS_EXECUTABLE value");
        let interpreter_config = get_config_from_interpreter(interpreter_path)?;

        return Ok((
            interpreter_config,
            fix_config_map(get_config_vars(interpreter_path)?),
        ));
    };

    // check default python
    let interpreter_path = "python";

    let interpreter_config = get_config_from_interpreter(interpreter_path)?;
    if interpreter_config.version.major == 3 {
        return Ok((
            interpreter_config,
            fix_config_map(get_config_vars(interpreter_path)?),
        ));
    }

    let major_interpreter_path = "python3";
    let interpreter_config = get_config_from_interpreter(major_interpreter_path)?;
    if interpreter_config.version.major == 3 {
        return Ok((
            interpreter_config,
            fix_config_map(get_config_vars(major_interpreter_path)?),
        ));
    }

    Err(format!("No python interpreter found"))
}

/// Extract compilation vars from the specified interpreter.
fn get_config_from_interpreter(interpreter: &str) -> Result<InterpreterConfig, String> {
    let script = r#"
import sys
import sysconfig
import platform
import json

PYPY = platform.python_implementation() == "PyPy"

try:
    base_prefix = sys.base_prefix
except AttributeError:
    base_prefix = sys.exec_prefix

print(json.dumps({
    "version": {
        "major": sys.version_info[0],
        "minor": sys.version_info[1],
        "implementation": platform.python_implementation()
    },
    "libdir": sysconfig.get_config_var('LIBDIR'),
    "ld_version": sysconfig.get_config_var('LDVERSION') or sysconfig.get_config_var('py_version_short'),
    "base_prefix": base_prefix,
    "shared": PYPY or bool(sysconfig.get_config_var('Py_ENABLE_SHARED'))
}))
"#;
    let json = run_python_script(interpreter, script)?;
    serde_json::from_str(&json).map_err(|e| format!("Deserializing failed: {}", e))
}

fn configure(interpreter_config: &InterpreterConfig) -> Result<(String), String> {
    if let Some(minor) = interpreter_config.version.minor {
        if minor < PY3_MIN_MINOR {
            return Err(format!(
                "Python 3 required version is 3.{}, current version is 3.{}",
                PY3_MIN_MINOR, minor
            ));
        }
    }

    let is_extension_module = env::var_os("CARGO_FEATURE_EXTENSION_MODULE").is_some();
    if !is_extension_module || cfg!(target_os = "windows") {
        println!(
            "{}",
            get_rustc_link_lib(
                &interpreter_config.version,
                &interpreter_config.ld_version,
                interpreter_config.shared
            )
            .unwrap()
        );
        if let Some(libdir) = &interpreter_config.libdir {
            println!("cargo:rustc-link-search=native={}", libdir);
        } else if cfg!(target_os = "windows") {
            println!(
                "cargo:rustc-link-search=native={}\\libs",
                interpreter_config.base_prefix
            );
        }
    }

    let mut flags = String::new();

    if interpreter_config.version.implementation == PythonInterpreterKind::PyPy {
        println!("cargo:rustc-cfg=PyPy");
        flags += format!("CFG_PyPy").as_ref();
    };

    if interpreter_config.version.major == 2 {
        // fail PYTHON_SYS_EXECUTABLE=python2 cargo ...
        return Err("Python 2 is not supported".to_string());
    }

    if env::var_os("CARGO_FEATURE_ABI3").is_some() {
        println!("cargo:rustc-cfg=Py_LIMITED_API");
    }

    if let Some(minor) = interpreter_config.version.minor {
        for i in 5..(minor + 1) {
            println!("cargo:rustc-cfg=Py_3_{}", i);
            flags += format!("CFG_Py_3_{},", i).as_ref();
        }
    }
    println!("cargo:rustc-cfg=Py_3");

    return Ok(flags);
}

fn check_rustc_version() {
    let ok_channel = supports_features();
    let ok_version = is_min_version(MIN_VERSION);
    let ok_date = is_min_date(MIN_DATE);

    let print_version_err = |version: &str, date: &str| {
        eprintln!(
            "Installed version is: {} ({}). Minimum required: {} ({}).",
            version, date, MIN_VERSION, MIN_DATE
        );
    };

    match (ok_channel, ok_version, ok_date) {
        (Some(ok_channel), Some((ok_version, version)), Some((ok_date, date))) => {
            if !ok_channel {
                eprintln!("Error: pyo3 requires a nightly or dev version of Rust.");
                print_version_err(&*version, &*date);
                panic!("Aborting compilation due to incompatible compiler.")
            }

            if !ok_version || !ok_date {
                eprintln!("Error: pyo3 requires a more recent version of rustc.");
                eprintln!("Use `rustup update` or your preferred method to update Rust");
                print_version_err(&*version, &*date);
                panic!("Aborting compilation due to incompatible compiler.")
            }
        }
        _ => {
            println!(
                "cargo:warning={}",
                "pyo3 was unable to check rustc compatibility."
            );
            println!(
                "cargo:warning={}",
                "Build may fail due to incompatible rustc version."
            );
        }
    }
}

fn main() -> Result<(), String> {
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
    let (interpreter_config, mut config_map) = if cross_compiling {
        load_cross_compile_info()?
    } else {
        find_interpreter_and_get_config()?
    };

    let flags;
    match configure(&interpreter_config) {
        Ok(val) => flags = val,
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    }

    // These flags need to be enabled manually for PyPy, because it does not expose
    // them in `sysconfig.get_config_vars()`
    if interpreter_config.version.implementation == PythonInterpreterKind::PyPy {
        config_map.insert("WITH_THREAD".to_owned(), "1".to_owned());
        config_map.insert("Py_USING_UNICODE".to_owned(), "1".to_owned());
        config_map.insert("Py_UNICODE_SIZE".to_owned(), "4".to_owned());
        config_map.insert("Py_UNICODE_WIDE".to_owned(), "1".to_owned());
    }

    // WITH_THREAD is always on for 3.7
    if interpreter_config.version.major == 3 && interpreter_config.version.minor.unwrap_or(0) >= 7 {
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

    Ok(())
}
