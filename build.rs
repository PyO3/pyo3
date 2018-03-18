extern crate regex;
extern crate version_check;

use std::process::Command;
use std::collections::HashMap;
use std::env;
use std::fmt;

use regex::Regex;
use version_check::{is_min_date, is_min_version, supports_features};
use std::path::{Path, PathBuf};

#[feature(match_default_bindings)]

// Specifies the minimum nightly version needed to compile pyo3.
const MIN_DATE: &'static str = "2017-11-07";
const MIN_VERSION: &'static str = "1.23.0-nightly";

fn canonicalize_executable<P>(exe_name: P) -> Option<PathBuf>
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
        try!(self.major.fmt(f));
        try!(f.write_str("."));
        match self.minor {
            Some(minor) => try!(minor.fmt(f)),
            None => try!(f.write_str("*")),
        };
        Ok(())
    }
}

#[derive(Debug)]
struct InterpreterConfig {
    version: PythonVersion,
    path: PathBuf,
    libpath: String,
    enable_shared: bool,
    ld_version: String,
    exec_prefix: String,
    is_pypy: bool,
}

impl InterpreterConfig {
    fn from_cpython(interpreter: PathBuf) -> Result<InterpreterConfig, String> {
        let script = "import sys; import sysconfig;\
         print(sys.version_info[0:2]); \
         print(sysconfig.get_config_var('LIBDIR')); \
         print(sysconfig.get_config_var('Py_ENABLE_SHARED')); \
         print(sysconfig.get_config_var('LDVERSION') or sysconfig.get_config_var('py_version_short')); \
         print(sys.exec_prefix);";

        let out = try!(run_python_script(&interpreter, script));

        let lines: Vec<&str> = out.lines().collect();

        let cpython_version = try!(parse_interpreter_version(&lines[0]));

        Ok(InterpreterConfig {
            version: cpython_version,
            path: interpreter,
            libpath: lines[1].to_string(),
            enable_shared: lines[2].parse().unwrap(),
            ld_version: lines[3].to_string(),
            exec_prefix: lines[4].to_string(),
            is_pypy: false,
        })
    }

    fn from_pypy(interpreter: PathBuf) -> Result<InterpreterConfig, String> {
        Ok(InterpreterConfig {
            version: PythonVersion {
                major: 3,
                minor: Some(5),
            },
            path: interpreter,
            libpath: "/Users/omerba/anaconda/lib".to_string(),
            enable_shared: true,
            ld_version: "3.5".to_string(),
            exec_prefix: "/Users/omerba/anaconda".to_string(),
            is_pypy: true,
        })
    }
}

const PY3_MIN_MINOR: u8 = 5;

const CFG_KEY: &'static str = "py_sys_config";

// windows' python writes out lines with the windows crlf sequence;
// posix platforms and mac os should write out lines with just lf.
#[cfg(target_os = "windows")]
static NEWLINE_SEQUENCE: &'static str = "\r\n";

#[cfg(not(target_os = "windows"))]
static NEWLINE_SEQUENCE: &'static str = "\n";

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

/// Examine python's compile flags to pass to cfg by launching
/// the interpreter and printing variables of interest from
/// sysconfig.get_config_vars.
#[cfg(not(target_os = "windows"))]
fn get_config_vars(python_path: &PathBuf) -> Result<HashMap<String, String>, String> {
    let mut script = "import sysconfig; \
                      config = sysconfig.get_config_vars();"
        .to_owned();

    for k in SYSCONFIG_FLAGS.iter().chain(SYSCONFIG_VALUES.iter()) {
        script.push_str(&format!(
            "print(config.get('{}', {}))",
            k,
            if is_value(k) { "None" } else { "0" }
        ));
        script.push_str(";");
    }

    let out = try!(run_python_script(python_path, &script));

    let split_stdout: Vec<&str> = out.trim_right().split(NEWLINE_SEQUENCE).collect();
    if split_stdout.len() != SYSCONFIG_VALUES.len() + SYSCONFIG_FLAGS.len() {
        return Err(format!(
            "python stdout len didn't return expected number of lines:
{}",
            split_stdout.len()
        ).to_string());
    }
    let all_vars = SYSCONFIG_FLAGS.iter().chain(SYSCONFIG_VALUES.iter());
    // let var_map: HashMap<String, String> = HashMap::new();
    let mut all_vars = all_vars.zip(split_stdout.iter()).fold(
        HashMap::new(),
        |mut memo: HashMap<String, String>, (&k, &v)| {
            if !(v.to_owned() == "None" && is_value(k)) {
                memo.insert(k.to_owned(), v.to_owned());
            }
            memo
        },
    );

    let debug = if let Some(val) = all_vars.get("Py_DEBUG") {
        val == "1"
    } else {
        false
    };
    if debug {
        all_vars.insert("Py_REF_DEBUG".to_owned(), "1".to_owned());
        all_vars.insert("Py_TRACE_REFS".to_owned(), "1".to_owned());
        all_vars.insert("COUNT_ALLOCS".to_owned(), "1".to_owned());
    }

    Ok(all_vars)
}

#[cfg(target_os = "windows")]
fn get_config_vars(_: &String) -> Result<HashMap<String, String>, String> {
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
fn run_python_script(interpreter_path: &PathBuf, script: &str) -> Result<String, String> {
    let mut cmd = Command::new(interpreter_path);
    cmd.arg("-c").arg(script);

    let out = try!(
        cmd.output()
            .map_err(|e| format!("failed to run python interpreter `{:?}`: {}", cmd, e))
    );

    if !out.status.success() {
        let stderr = String::from_utf8(out.stderr).unwrap();
        let mut msg = format!("python script failed with stderr:\n\n");
        msg.push_str(&stderr);
        return Err(msg);
    }

    let out = String::from_utf8(out.stdout).unwrap();
    return Ok(out);
}

#[cfg(not(target_os = "macos"))]
#[cfg(not(target_os = "windows"))]
fn get_rustc_link_lib(interpreter_config: &InterpreterConfig) -> Result<String, String> {
    if interpreter_config.enable_shared {
        Ok(format!(
            "cargo:rustc-link-lib=python{}",
            interpreter_config.ld_version
        ))
    } else {
        Ok(format!(
            "cargo:rustc-link-lib=static=python{}",
            interpreter_config.ld_version
        ))
    }
}

#[cfg(target_os = "windows")]
fn get_rustc_link_lib(interpreter_config: &InterpreterConfig) -> Result<String, String> {
    if interpreter_config.is_pypy {
        let link_library_name = match interpreter_config.version.major {
            2 => "pypy-c",
            3 => "pypy3-c",
        };
        // All modern PyPy versions with cpyext are compiled as shared libraries.
        return Ok(format!("cargo:rustc-link-lib={}", link_library_name));
    }

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

#[cfg(target_os = "macos")]
fn get_rustc_link_lib(interpreter_config: &InterpreterConfig) -> Result<String, String> {
    if interpreter_config.is_pypy {
        let link_library_name = match interpreter_config.version.major {
            2 => "pypy-c",
            3 => "pypy3-c",
            _ => unreachable!(),
        };
        // All modern PyPy versions with cpyext are compiled as shared libraries.
        return Ok(format!("cargo:rustc-link-lib={}", link_library_name));
    }

    // os x can be linked to a framework or static or dynamic, and
    // Py_ENABLE_SHARED is wrong; framework means shared library
    match get_macos_linkmodel(&interpreter_config.path)
        .unwrap()
        .as_ref()
    {
        "static" => Ok(format!(
            "cargo:rustc-link-lib=static=python{}",
            interpreter_config.ld_version
        )),
        "shared" => Ok(format!(
            "cargo:rustc-link-lib=python{}",
            interpreter_config.ld_version
        )),
        "framework" => Ok(format!(
            "cargo:rustc-link-lib=python{}",
            interpreter_config.ld_version
        )),
        other => Err(format!("unknown linkmodel {}", other)),
    }
}

#[cfg(target_os = "macos")]
fn get_macos_linkmodel(interpreter_path: &PathBuf) -> Result<String, String> {
    let script = "import sysconfig; print('framework' if sysconfig.get_config_var('PYTHONFRAMEWORK') else ('shared' if sysconfig.get_config_var('Py_ENABLE_SHARED') else 'static'));";
    let out = run_python_script(interpreter_path, script).unwrap();
    Ok(out.trim_right().to_owned())
}

/// Parse string as interpreter version.
fn parse_interpreter_version(line: &str) -> Result<PythonVersion, String> {
    let version_re = Regex::new(r"\((\d+), (\d+)\)").unwrap();
    match version_re.captures(&line) {
        Some(cap) => Ok(PythonVersion {
            major: cap.get(1).unwrap().as_str().parse().unwrap(),
            minor: Some(cap.get(2).unwrap().as_str().parse().unwrap()),
        }),
        None => Err(format!("Unexpected response to version query {}", line)),
    }
}

fn check_pypy(interpreter_path: &PathBuf) -> bool {
    let script = "import sys; print('__pypy__' in sys.builtin_module_names)";
    let is_pypy: bool = run_python_script(interpreter_path, script)
        .unwrap()
        .to_lowercase()
        .trim_right()
        .parse()
        .unwrap();
    return is_pypy;
}

/// Locate a suitable python interpreter and extract config from it.
/// If the environment variable `PYTHON_SYS_EXECUTABLE`, use the provided
/// path a Python executable, and raises an error if the version doesn't match.
/// Else tries to execute the interpreter as "python", "python{major version}",
/// "python{major version}.{minor version}" in order until one
/// is of the version we are expecting.
fn find_interpreter(expected_version: &PythonVersion) -> Result<InterpreterConfig, String> {
    // To use PyPy, a valid pypy executable must be passed to PYTHON_SYS_EXECUTABLE
    if let Some(interpreter_from_env) = env::var_os("PYTHON_SYS_EXECUTABLE") {
        let interpreter_path_or_executable = interpreter_from_env
            .to_str()
            .expect("Unable to get PYTHON_SYS_EXECUTABLE value");

        let interpreter_path =
            canonicalize_executable(interpreter_path_or_executable).expect(&format!(
                "Could not find interpreter passed in PYTHON_SYS_EXECUTABLE={}\n",
                interpreter_path_or_executable
            ));

        let interpreter_config: InterpreterConfig;

        if check_pypy(&interpreter_path) {
            interpreter_config = try!(InterpreterConfig::from_pypy(interpreter_path));
        } else {
            interpreter_config = try!(InterpreterConfig::from_cpython(interpreter_path));
        }

        if expected_version == &interpreter_config.version {
            return Ok(interpreter_config);
        } else {
            return Err(format!(
                "Unsupported python version in PYTHON_SYS_EXECUTABLE={}\n\
                 \tmin version {} != found {}",
                interpreter_path_or_executable, expected_version, interpreter_config.version
            ));
        }
    }

    // check default python
    let system_python = "python";

    let mut possible_python_paths = vec![
        "python".to_string(),
        format!("python{}", expected_version.major),
    ];
    if let Some(minor) = expected_version.minor {
        possible_python_paths.push(format!("python{}.{}", expected_version.major, minor))
    }

    for possible_path in possible_python_paths {
        let interpreter_path = canonicalize_executable(&possible_path);
        if interpreter_path.is_some() {
            let interpreter_config =
                try!(InterpreterConfig::from_cpython(interpreter_path.unwrap()));
            if expected_version == &interpreter_config.version {
                return Ok(interpreter_config);
            }
        }
    }

    Err(format!("No python interpreter found"))
}

/// Deduce configuration from the 'python' in the current PATH and print
/// cargo vars to stdout.
///
/// Note that if the python doesn't satisfy expected_version, this will error.
fn emit_cargo_vars_from_configuration(
    interpreter_config: &InterpreterConfig,
) -> Result<String, String> {
    let is_extension_module = env::var_os("CARGO_FEATURE_EXTENSION_MODULE").is_some();

    if interpreter_config.is_pypy {
        println!("cargo:rustc-cfg=PyPy");
    }

    if !is_extension_module || cfg!(target_os = "windows") {
        println!("{:?}", get_rustc_link_lib(&interpreter_config));

        if interpreter_config.libpath != "None" {
            println!(
                "cargo:rustc-link-search=native={}",
                interpreter_config.libpath
            );
        } else if cfg!(target_os = "windows") {
            println!(
                "cargo:rustc-link-search=native={}\\libs",
                interpreter_config.exec_prefix
            );
        }
    }

    let mut flags = String::new();

    if let PythonVersion {
        major: 3,
        minor: some_minor,
    } = interpreter_config.version
    {
        if env::var_os("CARGO_FEATURE_PEP_384").is_some() {
            println!("cargo:rustc-cfg=Py_LIMITED_API");
        }
        if let Some(minor) = some_minor {
            if minor < PY3_MIN_MINOR {
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
fn version_from_env() -> Result<PythonVersion, String> {
    let re = Regex::new(r"CARGO_FEATURE_PYTHON(\d+)(_(\d+))?").unwrap();
    // sort env::vars so we get more explicit version specifiers first
    // so if the user passes e.g. the python-3 feature and the python-3-5
    // feature, python-3-5 takes priority.
    let mut vars = env::vars().collect::<Vec<_>>();

    vars.sort_by(|a, b| b.cmp(a));
    for (key, _) in vars {
        match re.captures(&key) {
            Some(cap) => {
                return Ok(PythonVersion {
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

    Err(
        "Python version feature was not found. At least one python version \
         feature must be enabled."
            .to_owned(),
    )
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

#[cfg(test)]
mod test {
    use std::env;
    use {find_interpreter, InterpreterConfig, PythonVersion};

    #[test]
    fn test_correctly_detects_cpython() {
        let python_version_major_only = PythonVersion {
            major: 3,
            minor: None,
        };
        let expected_config = InterpreterConfig {
            version: python_version_major_only,
            path: ("bla"),
            libpath: String::from("bla"),
            enable_shared: false,
            ld_version: String::from("bla"),
            exec_prefix: String::from("bla"),
            is_pypy: false,
        };

        assert_eq!(
            find_interpreter(&python_version_major_only).unwrap(),
            expected_config
        )
    }
}
