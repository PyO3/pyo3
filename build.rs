use regex::Regex;
use serde::Deserialize;
use std::io::{self, BufRead, BufReader};
use std::process::{Command, Stdio};
use std::{collections::HashMap, convert::AsRef, env, fmt, fs::File, path::Path};
use version_check::{Channel, Date, Version};

/// Specifies the minimum nightly version needed to compile pyo3.
/// Keep this synced up with the travis ci config,
/// But note that this is the rustc version which can be lower than the nightly version
const MIN_DATE: &str = "2020-01-20";
const MIN_VERSION: &str = "1.42.0-nightly";

const PY3_MIN_MINOR: u8 = 5;
const CFG_KEY: &str = "py_sys_config";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// A simple macro for returning an error. Resembles failure::bail and anyhow::bail.
macro_rules! bail {
    ($msg: expr) => { return Err($msg.into()); };
    ($fmt: literal $(, $args: expr)+) => { return Err(format!($fmt $(,$args)+).into()); };
}

/// Information returned from python interpreter
#[derive(Deserialize, Debug)]
struct InterpreterConfig {
    version: PythonVersion,
    libdir: Option<String>,
    shared: bool,
    ld_version: String,
    /// Prefix used for determining the directory of libpython
    base_prefix: String,
    executable: String,
    calcsize_pointer: Option<u32>,
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.major.fmt(f)?;
        f.write_str(".")?;
        match self.minor {
            Some(minor) => minor.fmt(f)?,
            None => f.write_str("*")?,
        };
        Ok(())
    }
}

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
static SYSCONFIG_FLAGS: [&str; 7] = [
    "Py_USING_UNICODE",
    "Py_UNICODE_WIDE",
    "WITH_THREAD",
    "Py_DEBUG",
    "Py_REF_DEBUG",
    "Py_TRACE_REFS",
    "COUNT_ALLOCS",
];

static SYSCONFIG_VALUES: [&str; 1] = [
    // cfg doesn't support flags with values, just bools - so flags
    // below are translated into bools as {varname}_{val}
    //
    // for example, Py_UNICODE_SIZE_2 or Py_UNICODE_SIZE_4
    "Py_UNICODE_SIZE", // note - not present on python 3.3+, which is always wide
];

/// Attempts to parse the header at the given path, returning a map of definitions to their values.
/// Each entry in the map directly corresponds to a `#define` in the given header.
fn parse_header_defines(header_path: impl AsRef<Path>) -> Result<HashMap<String, String>> {
    // This regex picks apart a C style, single line `#define` statement into an identifier and a
    // value. e.g. for the line `#define Py_DEBUG 1`, this regex will capture `Py_DEBUG` into
    // `ident` and `1` into `value`.
    let define_regex = Regex::new(r"^\s*#define\s+(?P<ident>[a-zA-Z0-9_]+)\s+(?P<value>.+)\s*$")?;

    let header_file = File::open(header_path.as_ref())?;
    let header_reader = BufReader::new(&header_file);

    let mut definitions = HashMap::new();
    let tostr = |r: regex::Match<'_>| r.as_str().to_string();
    for maybe_line in header_reader.lines() {
        if let Some(captures) = define_regex.captures(&maybe_line?) {
            match (captures.name("ident"), captures.name("value")) {
                (Some(key), Some(val)) => definitions.insert(tostr(key), tostr(val)),
                _ => None,
            };
        }
    }
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

fn load_cross_compile_info() -> Result<(InterpreterConfig, HashMap<String, String>)> {
    let python_include_dir = env::var("PYO3_CROSS_INCLUDE_DIR")?;
    let python_include_dir = Path::new(&python_include_dir);

    let patchlevel_defines = parse_header_defines(python_include_dir.join("patchlevel.h"))?;

    let major = match patchlevel_defines
        .get("PY_MAJOR_VERSION")
        .map(|major| major.parse::<u8>())
    {
        Some(Ok(major)) => major,
        Some(Err(e)) => bail!("Failed to parse PY_MAJOR_VERSION: {}", e),
        None => bail!("PY_MAJOR_VERSION undefined"),
    };

    let minor = match patchlevel_defines
        .get("PY_MINOR_VERSION")
        .map(|minor| minor.parse::<u8>())
    {
        Some(Ok(minor)) => minor,
        Some(Err(e)) => bail!("Failed to parse PY_MINOR_VERSION: {}", e),
        None => bail!("PY_MINOR_VERSION undefined"),
    };

    let python_version = PythonVersion {
        major,
        minor: Some(minor),
        implementation: PythonInterpreterKind::CPython,
    };

    let config_map = parse_header_defines(python_include_dir.join("pyconfig.h"))?;
    let shared = match config_map
        .get("Py_ENABLE_SHARED")
        .map(|x| x.as_str())
        .ok_or("Py_ENABLE_SHARED is not defined")?
    {
        "1" | "true" | "True" => true,
        "0" | "false" | "False" => false,
        _ => panic!("Py_ENABLE_SHARED must be a bool (1/true/True or 0/false/False"),
    };

    let interpreter_config = InterpreterConfig {
        version: python_version,
        libdir: Some(env::var("PYO3_CROSS_LIB_DIR")?),
        shared,
        ld_version: "".to_string(),
        base_prefix: "".to_string(),
        executable: "".to_string(),
        calcsize_pointer: None,
    };

    Ok((interpreter_config, fix_config_map(config_map)))
}

/// Examine python's compile flags to pass to cfg by launching
/// the interpreter and printing variables of interest from
/// sysconfig.get_config_vars.
#[cfg(not(target_os = "windows"))]
fn get_config_vars(python_path: &str) -> Result<HashMap<String, String>> {
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
        bail!(
            "Python stdout len didn't return expected number of lines: {}",
            split_stdout.len()
        );
    }
    let all_vars = SYSCONFIG_FLAGS.iter().chain(SYSCONFIG_VALUES.iter());
    let all_vars = all_vars
        .zip(split_stdout.iter())
        .fold(HashMap::new(), |mut memo, (&k, &v)| {
            if !(v == "None" && is_value(k)) {
                memo.insert(k.to_string(), v.to_string());
            }
            memo
        });

    Ok(fix_config_map(all_vars))
}

#[cfg(target_os = "windows")]
fn get_config_vars(_: &str) -> Result<HashMap<String, String>> {
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
    Ok(fix_config_map(map))
}

fn is_value(key: &str) -> bool {
    SYSCONFIG_VALUES.iter().any(|x| *x == key)
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
fn run_python_script(interpreter: &str, script: &str) -> Result<String> {
    let out = Command::new(interpreter)
        .args(&["-c", script])
        .stderr(Stdio::inherit())
        .output();

    match out {
        Err(err) => {
            if err.kind() == io::ErrorKind::NotFound {
                bail!(
                    "Could not find any interpreter at {}, \
                     are you sure you have Python installed on your PATH?",
                    interpreter
                );
            } else {
                bail!(
                    "Failed to run the Python interpreter at {}: {}",
                    interpreter,
                    err
                );
            }
        }
        Ok(ok) if !ok.status.success() => bail!("Python script failed: {}"),
        Ok(ok) => Ok(String::from_utf8(ok.stdout)?),
    }
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
fn get_rustc_link_lib(config: &InterpreterConfig) -> Result<String> {
    if config.shared {
        Ok(format!(
            "cargo:rustc-link-lib={}",
            get_library_link_name(&config.version, &config.ld_version)
        ))
    } else {
        Ok(format!(
            "cargo:rustc-link-lib=static={}",
            get_library_link_name(&config.version, &config.ld_version)
        ))
    }
}

#[cfg(target_os = "macos")]
fn get_macos_linkmodel(config: &InterpreterConfig) -> Result<String> {
    let script = r#"
import sysconfig

if sysconfig.get_config_var("PYTHONFRAMEWORK"):
    print("framework")
elif sysconfig.get_config_var("Py_ENABLE_SHARED"):
    print("shared")
else:
    print("static")
"#;
    let out = run_python_script(&config.executable, script)?;
    Ok(out.trim_end().to_owned())
}

#[cfg(target_os = "macos")]
fn get_rustc_link_lib(config: &InterpreterConfig) -> Result<String> {
    // os x can be linked to a framework or static or dynamic, and
    // Py_ENABLE_SHARED is wrong; framework means shared library
    match get_macos_linkmodel(config)?.as_ref() {
        "static" => Ok(format!(
            "cargo:rustc-link-lib=static={}",
            get_library_link_name(&config.version, &config.ld_version)
        )),
        "shared" => Ok(format!(
            "cargo:rustc-link-lib={}",
            get_library_link_name(&config.version, &config.ld_version)
        )),
        "framework" => Ok(format!(
            "cargo:rustc-link-lib={}",
            get_library_link_name(&config.version, &config.ld_version)
        )),
        other => bail!("unknown linkmodel {}", other),
    }
}

#[cfg(target_os = "windows")]
fn get_rustc_link_lib(config: &InterpreterConfig) -> Result<String> {
    // Py_ENABLE_SHARED doesn't seem to be present on windows.
    Ok(format!(
        "cargo:rustc-link-lib=pythonXY:{}",
        get_library_link_name(&config.version, &config.ld_version)
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
fn find_interpreter_and_get_config() -> Result<(InterpreterConfig, HashMap<String, String>)> {
    if let Some(sys_executable) = env::var_os("PYTHON_SYS_EXECUTABLE") {
        let interpreter_path = sys_executable
            .to_str()
            .ok_or("Unable to get PYTHON_SYS_EXECUTABLE value")?;
        let interpreter_config = get_config_from_interpreter(interpreter_path)?;

        return Ok((interpreter_config, get_config_vars(interpreter_path)?));
    };

    let python_interpreter = ["python", "python3"]
        .iter()
        .find(|bin| {
            if let Ok(out) = Command::new(bin).arg("--version").output() {
                // begin with `Python 3.X.X :: additional info`
                out.stdout.starts_with(b"Python 3") || out.stderr.starts_with(b"Python 3")
            } else {
                false
            }
        })
        .ok_or("Python 3.x interpreter not found")?;

    // check default python
    let interpreter_config = get_config_from_interpreter(&python_interpreter)?;
    if interpreter_config.version.major == 3 {
        return Ok((interpreter_config, get_config_vars(&python_interpreter)?));
    }

    let interpreter_config = get_config_from_interpreter(&python_interpreter)?;
    if interpreter_config.version.major == 3 {
        return Ok((interpreter_config, get_config_vars(&python_interpreter)?));
    }

    Err("No Python interpreter found".into())
}

/// Extract compilation vars from the specified interpreter.
fn get_config_from_interpreter(interpreter: &str) -> Result<InterpreterConfig> {
    let script = r#"
import json
import platform
import struct
import sys
import sysconfig

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
    "shared": PYPY or bool(sysconfig.get_config_var('Py_ENABLE_SHARED')),
    "executable": sys.executable,
    "calcsize_pointer": struct.calcsize("P"),
}))
"#;
    let json = run_python_script(interpreter, script)?;
    Ok(serde_json::from_str(&json)
        .map_err(|e| format!("Failed to get InterPreterConfig: {}", e))?)
}

fn configure(interpreter_config: &InterpreterConfig) -> Result<String> {
    if let Some(minor) = interpreter_config.version.minor {
        if minor < PY3_MIN_MINOR {
            bail!(
                "Python 3 required version is 3.{}, current version is 3.{}",
                PY3_MIN_MINOR,
                minor
            );
        }
    }

    check_target_architecture(interpreter_config)?;

    let is_extension_module = env::var_os("CARGO_FEATURE_EXTENSION_MODULE").is_some();
    if !is_extension_module || cfg!(target_os = "windows") {
        println!("{}", get_rustc_link_lib(&interpreter_config)?);
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
        flags += "CFG_PyPy";
    };

    if interpreter_config.version.major == 2 {
        // fail PYTHON_SYS_EXECUTABLE=python2 cargo ...
        bail!("Python 2 is not supported");
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

    Ok(flags)
}

fn check_target_architecture(interpreter_config: &InterpreterConfig) -> Result<()> {
    // Try to check whether the target architecture matches the python library
    let rust_target = match env::var("CARGO_CFG_TARGET_POINTER_WIDTH")?.as_str() {
        "64" => "64-bit",
        "32" => "32-bit",
        x => bail!("unexpected Rust target pointer width: {}", x),
    };

    // The reason we don't use platform.architecture() here is that it's not
    // reliable on macOS. See https://stackoverflow.com/a/1405971/823869.
    // Similarly, sys.maxsize is not reliable on Windows. See
    // https://stackoverflow.com/questions/1405913/how-do-i-determine-if-my-python-shell-is-executing-in-32bit-or-64bit-mode-on-os/1405971#comment6209952_1405971
    // and https://stackoverflow.com/a/3411134/823869.
    let python_target = match interpreter_config.calcsize_pointer {
        Some(8) => "64-bit",
        Some(4) => "32-bit",
        None => {
            // Unset, e.g. because we're cross-compiling. Don't check anything
            // in this case.
            return Ok(());
        }
        Some(n) => bail!("unexpected Python calcsize_pointer value: {}", n),
    };

    if rust_target != python_target {
        bail!(
            "Your Rust target architecture ({}) does not match your python interpreter ({})",
            rust_target,
            python_target
        );
    }

    Ok(())
}

fn check_rustc_version() -> Result<()> {
    let channel = Channel::read().ok_or("Failed to determine rustc channel")?;
    if !channel.supports_features() {
        bail!("PyO3 requires a nightly or dev version of Rust.");
    }

    let actual_version = Version::read().ok_or("Failed to determine the rustc version")?;
    if !actual_version.at_least(MIN_VERSION) {
        bail!(
            "PyO3 requires at least rustc {}, while the current version is {}",
            MIN_VERSION,
            actual_version
        )
    }

    let actual_date = Date::read().ok_or("Failed to determine the rustc date")?;
    if !actual_date.at_least(MIN_DATE) {
        bail!(
            "PyO3 requires at least rustc {}, while the current rustc date is {}",
            MIN_DATE,
            actual_date
        )
    }
    Ok(())
}

fn main() -> Result<()> {
    check_rustc_version()?;
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

    let flags = configure(&interpreter_config)?;

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
        if let Some(line) = cfg_line_for_var(key, val) {
            println!("{}", line)
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
    let flags = config_map.iter().fold("".to_owned(), |memo, (key, val)| {
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
        if !flags.is_empty() {
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
