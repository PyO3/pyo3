use regex::Regex;
use std::collections::HashMap;
use std::convert::AsRef;
use std::env;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Command;
use std::process::Stdio;
use version_check::{is_min_date, is_min_version, supports_features};

// Specifies the minimum nightly version needed to compile pyo3.
// This requirement is due to https://github.com/rust-lang/rust/issues/55380
const MIN_DATE: &'static str = "2018-11-02";
const MIN_VERSION: &'static str = "1.32.0-nightly";

#[derive(Debug)]
struct PythonVersion {
    major: u8,
    // minor == None means any minor version will do
    minor: Option<u8>,
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

fn load_cross_compile_info() -> Result<(PythonVersion, HashMap<String, String>, Vec<String>), String>
{
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
    };

    let config_map = parse_header_defines(python_include_dir.join("pyconfig.h"))?;

    let config_lines = vec![
        "".to_owned(), // compatibility, not used when cross compiling.
        env::var("PYO3_CROSS_LIB_DIR").unwrap(),
        config_map
            .get("Py_ENABLE_SHARED")
            .expect("Py_ENABLE_SHARED undefined")
            .to_owned(),
        format!("{}.{}", major, minor),
        "".to_owned(), // compatibility, not used when cross compiling.
    ];

    Ok((python_version, fix_config_map(config_map), config_lines))
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
        .output()
        .map_err(|e| {
            format!(
                "Failed to run the python interpreter at {}: {}",
                interpreter, e
            )
        })?;

    if !out.status.success() {
        return Err(format!("python script failed"));
    }

    Ok(String::from_utf8(out.stdout).unwrap())
}

#[cfg(not(target_os = "macos"))]
#[cfg(not(target_os = "windows"))]
fn get_rustc_link_lib(
    _: &PythonVersion,
    ld_version: &str,
    enable_shared: bool,
) -> Result<String, String> {
    if enable_shared {
        Ok(format!("cargo:rustc-link-lib=python{}", ld_version))
    } else {
        Ok(format!("cargo:rustc-link-lib=static=python{}", ld_version))
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
fn get_rustc_link_lib(_: &PythonVersion, ld_version: &str, _: bool) -> Result<String, String> {
    // os x can be linked to a framework or static or dynamic, and
    // Py_ENABLE_SHARED is wrong; framework means shared library
    match get_macos_linkmodel().unwrap().as_ref() {
        "static" => Ok(format!("cargo:rustc-link-lib=static=python{}", ld_version)),
        "shared" => Ok(format!("cargo:rustc-link-lib=python{}", ld_version)),
        "framework" => Ok(format!("cargo:rustc-link-lib=python{}", ld_version)),
        other => Err(format!("unknown linkmodel {}", other)),
    }
}

/// Parse string as interpreter version.
fn get_interpreter_version(line: &str) -> Result<PythonVersion, String> {
    let version_re = Regex::new(r"\((\d+), (\d+)\)").unwrap();
    match version_re.captures(&line) {
        Some(cap) => Ok(PythonVersion {
            major: cap.get(1).unwrap().as_str().parse().unwrap(),
            minor: Some(cap.get(2).unwrap().as_str().parse().unwrap()),
        }),
        None => Err(format!("Unexpected response to version query {}", line)),
    }
}

#[cfg(target_os = "windows")]
fn get_rustc_link_lib(version: &PythonVersion, _: &str, _: bool) -> Result<String, String> {
    // Py_ENABLE_SHARED doesn't seem to be present on windows.
    Ok(format!(
        "cargo:rustc-link-lib=pythonXY:python{}{}",
        version.major,
        match version.minor {
            Some(minor) => minor.to_string(),
            None => "".to_owned(),
        }
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
fn find_interpreter_and_get_config(
) -> Result<(PythonVersion, HashMap<String, String>, Vec<String>), String> {
    let version = version_from_env();

    if let Some(sys_executable) = env::var_os("PYTHON_SYS_EXECUTABLE") {
        let interpreter_path = sys_executable
            .to_str()
            .expect("Unable to get PYTHON_SYS_EXECUTABLE value");
        let (interpreter_version, lines) = get_config_from_interpreter(interpreter_path)?;
        if version != None && version.as_ref().unwrap() != &interpreter_version {
            panic!(
                "Unsupported python version in PYTHON_SYS_EXECUTABLE={}\n\
                 \tmin version {} != found {}",
                interpreter_path,
                version.unwrap(),
                interpreter_version
            );
        } else {
            return Ok((
                interpreter_version,
                fix_config_map(get_config_vars(interpreter_path)?),
                lines,
            ));
        }
    };

    let expected_version = version.unwrap_or(PythonVersion {
        major: 3,
        minor: None,
    });

    // check default python
    let interpreter_path = "python";
    let (interpreter_version, lines) = get_config_from_interpreter(interpreter_path)?;
    if expected_version == interpreter_version {
        return Ok((
            interpreter_version,
            fix_config_map(get_config_vars(interpreter_path)?),
            lines,
        ));
    }

    let major_interpreter_path = &format!("python{}", expected_version.major);
    let (interpreter_version, lines) = get_config_from_interpreter(major_interpreter_path)?;
    if expected_version == interpreter_version {
        return Ok((
            interpreter_version,
            fix_config_map(get_config_vars(major_interpreter_path)?),
            lines,
        ));
    }

    if let Some(minor) = expected_version.minor {
        let minor_interpreter_path = &format!("python{}.{}", expected_version.major, minor);
        let (interpreter_version, lines) = get_config_from_interpreter(minor_interpreter_path)?;
        if expected_version == interpreter_version {
            return Ok((
                interpreter_version,
                fix_config_map(get_config_vars(minor_interpreter_path)?),
                lines,
            ));
        }
    }

    Err(format!("No python interpreter found"))
}

/// Extract compilation vars from the specified interpreter.
fn get_config_from_interpreter(interpreter: &str) -> Result<(PythonVersion, Vec<String>), String> {
    let script = r#"
import sys
import sysconfig

print(sys.version_info[0:2])
print(sysconfig.get_config_var('LIBDIR'))
print(sysconfig.get_config_var('Py_ENABLE_SHARED'))
print(sysconfig.get_config_var('LDVERSION') or sysconfig.get_config_var('py_version_short'))
print(sys.exec_prefix)
"#;
    let out = run_python_script(interpreter, script)?;
    let lines: Vec<String> = out.lines().map(|line| line.to_owned()).collect();
    let interpreter_version = get_interpreter_version(&lines[0])?;
    Ok((interpreter_version, lines))
}

fn configure(interpreter_version: &PythonVersion, lines: Vec<String>) -> Result<(String), String> {
    let libpath: &str = &lines[1];
    let enable_shared: &str = &lines[2];
    let ld_version: &str = &lines[3];
    let exec_prefix: &str = &lines[4];

    let is_extension_module = env::var_os("CARGO_FEATURE_EXTENSION_MODULE").is_some();
    if !is_extension_module || cfg!(target_os = "windows") {
        println!(
            "{}",
            get_rustc_link_lib(&interpreter_version, ld_version, enable_shared == "1").unwrap()
        );
        if libpath != "None" {
            println!("cargo:rustc-link-search=native={}", libpath);
        } else if cfg!(target_os = "windows") {
            println!("cargo:rustc-link-search=native={}\\libs", exec_prefix);
        }
    }

    let mut flags = String::new();

    if let PythonVersion {
        major: 3,
        minor: some_minor,
    } = interpreter_version
    {
        if env::var_os("CARGO_FEATURE_ABI3").is_some() {
            println!("cargo:rustc-cfg=Py_LIMITED_API");
        }
        if let Some(minor) = some_minor {
            if minor < &PY3_MIN_MINOR {
                return Err(format!(
                    "Python 3 required version is 3.{}, current version is 3.{}",
                    PY3_MIN_MINOR, minor
                ));
            }
            for i in 5..(minor + 1) {
                println!("cargo:rustc-cfg=Py_3_{}", i);
                flags += format!("CFG_Py_3_{},", i).as_ref();
            }
            println!("cargo:rustc-cfg=Py_3");
        }
    } else {
        println!("cargo:rustc-cfg=Py_2");
        flags += format!("CFG_Py_2,").as_ref();
    }
    return Ok(flags);
}

/// Determine the python version we're supposed to be building
/// from the features passed via the environment.
///
/// The environment variable can choose to omit a minor
/// version if the user doesn't care.
fn version_from_env() -> Option<PythonVersion> {
    let re = Regex::new(r"CARGO_FEATURE_PYTHON(\d+)(_(\d+))?").unwrap();
    // sort env::vars so we get more explicit version specifiers first
    // so if the user passes e.g. the python-3 feature and the python-3-5
    // feature, python-3-5 takes priority.
    let mut vars = env::vars().collect::<Vec<_>>();
    vars.sort_by(|a, b| b.cmp(a));
    for (key, _) in vars {
        match re.captures(&key) {
            Some(cap) => {
                return Some(PythonVersion {
                    major: cap.get(1).unwrap().as_str().parse().unwrap(),
                    minor: match cap.get(3) {
                        Some(s) => Some(s.as_str().parse().unwrap()),
                        None => None,
                    },
                });
            }
            None => (),
        }
    }
    None
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
    let (interpreter_version, mut config_map, lines) = if cross_compiling {
        load_cross_compile_info()
    } else {
        find_interpreter_and_get_config()
    }
    .unwrap();

    let flags = configure(&interpreter_version, lines).unwrap();

    // WITH_THREAD is always on for 3.7
    if interpreter_version.major == 3 && interpreter_version.minor.unwrap_or(0) >= 7 {
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
}
