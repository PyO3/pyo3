use std::{
    collections::HashMap,
    convert::AsRef,
    env, fmt,
    fs::{self, DirEntry, File},
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    str::FromStr,
};

const PY3_MIN_MINOR: u8 = 5;
const CFG_KEY: &str = "py_sys_config";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// A simple macro for returning an error. Resembles failure::bail and anyhow::bail.
macro_rules! bail {
    ($msg: expr) => { return Err($msg.into()); };
    ($fmt: literal $(, $args: expr)+) => { return Err(format!($fmt $(,$args)+).into()); };
}

/// Information returned from python interpreter
#[derive(Debug)]
struct InterpreterConfig {
    version: PythonVersion,
    libdir: Option<String>,
    shared: bool,
    ld_version: String,
    /// Prefix used for determining the directory of libpython
    base_prefix: String,
    executable: PathBuf,
    calcsize_pointer: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PythonInterpreterKind {
    CPython,
    PyPy,
}

#[derive(Debug, Clone)]
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

impl FromStr for PythonInterpreterKind {
    type Err = Box<dyn std::error::Error>;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "CPython" => Ok(PythonInterpreterKind::CPython),
            "PyPy" => Ok(PythonInterpreterKind::PyPy),
            _ => Err(format!("Invalid interpreter: {}", s).into()),
        }
    }
}

trait GetPrimitive {
    fn get_bool(&self, key: &str) -> Result<bool>;
    fn get_numeric<T: FromStr>(&self, key: &str) -> Result<T>;
}

impl GetPrimitive for HashMap<String, String> {
    fn get_bool(&self, key: &str) -> Result<bool> {
        match self
            .get(key)
            .map(|x| x.as_str())
            .ok_or(format!("{} is not defined", key))?
        {
            "1" | "true" | "True" => Ok(true),
            "0" | "false" | "False" => Ok(false),
            _ => Err(format!("{} must be a bool (1/true/True or 0/false/False", key).into()),
        }
    }

    fn get_numeric<T: FromStr>(&self, key: &str) -> Result<T> {
        self.get(key)
            .ok_or(format!("{} is not defined", key))?
            .parse::<T>()
            .map_err(|_| format!("Could not parse value of {}", key).into())
    }
}

struct CrossCompileConfig {
    lib_dir: PathBuf,
    include_dir: Option<PathBuf>,
    version: Option<String>,
    os: String,
    arch: String,
}

impl CrossCompileConfig {
    fn both() -> Result<Self> {
        Ok(CrossCompileConfig {
            include_dir: Some(CrossCompileConfig::validate_variable(
                "PYO3_CROSS_INCLUDE_DIR",
            )?),
            ..CrossCompileConfig::lib_only()?
        })
    }

    fn lib_only() -> Result<Self> {
        Ok(CrossCompileConfig {
            lib_dir: CrossCompileConfig::validate_variable("PYO3_CROSS_LIB_DIR")?,
            include_dir: None,
            os: env::var("CARGO_CFG_TARGET_OS").unwrap(),
            arch: env::var("CARGO_CFG_TARGET_ARCH").unwrap(),
            version: env::var_os("PYO3_CROSS_PYTHON_VERSION").map(|s| s.into_string().unwrap()),
        })
    }

    fn validate_variable(var: &str) -> Result<PathBuf> {
        let path = match env::var_os(var) {
            Some(v) => v,
            None => bail!(
                "Must provide {} environment variable when cross-compiling",
                var
            ),
        };

        if fs::metadata(&path).is_err() {
            bail!("{} value of {:?} does not exist", var, path)
        }

        Ok(path.into())
    }
}

fn cross_compiling() -> Result<Option<CrossCompileConfig>> {
    let target = env::var("TARGET")?;
    let host = env::var("HOST")?;
    if target == host || (target == "i686-pc-windows-msvc" && host == "x86_64-pc-windows-msvc") {
        return Ok(None);
    }

    if env::var("CARGO_CFG_TARGET_FAMILY")? == "windows" {
        Ok(Some(CrossCompileConfig::both()?))
    } else {
        Ok(Some(CrossCompileConfig::lib_only()?))
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
    let header_reader = BufReader::new(File::open(header_path.as_ref())?);
    let mut definitions = HashMap::new();
    for maybe_line in header_reader.lines() {
        let line = maybe_line?;
        let mut i = line.trim().split_whitespace();
        if i.next() == Some("#define") {
            if let (Some(key), Some(value), None) = (i.next(), i.next(), i.next()) {
                definitions.insert(key.into(), value.into());
            }
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

fn parse_script_output(output: &str) -> HashMap<String, String> {
    output
        .lines()
        .filter_map(|line| {
            let mut i = line.splitn(2, ' ');
            Some((i.next()?.into(), i.next()?.into()))
        })
        .collect()
}

/// Parse sysconfigdata file
///
/// The sysconfigdata is simply a dictionary containing all the build time variables used for the
/// python executable and library. Here it is read and added to a script to extract only what is
/// necessary. This necessitates a python interpreter for the host machine to work.
fn parse_sysconfigdata(config_path: impl AsRef<Path>) -> Result<HashMap<String, String>> {
    let mut script = fs::read_to_string(config_path)?;
    script += r#"
print("version_major", build_time_vars["VERSION"][0])  # 3
print("version_minor", build_time_vars["VERSION"][2])  # E.g., 8
if "WITH_THREAD" in build_time_vars:
    print("WITH_THREAD", build_time_vars["WITH_THREAD"])
if "Py_TRACE_REFS" in build_time_vars:
    print("Py_TRACE_REFS", build_time_vars["Py_TRACE_REFS"])
if "COUNT_ALLOCS" in build_time_vars:
    print("COUNT_ALLOCS", build_time_vars["COUNT_ALLOCS"])
if "Py_REF_DEBUG" in build_time_vars:
    print("Py_REF_DEBUG", build_time_vars["Py_REF_DEBUG"])
print("Py_DEBUG", build_time_vars["Py_DEBUG"])
print("Py_ENABLE_SHARED", build_time_vars["Py_ENABLE_SHARED"])
print("LDVERSION", build_time_vars["LDVERSION"])
print("SIZEOF_VOID_P", build_time_vars["SIZEOF_VOID_P"])
"#;
    let output = run_python_script(&find_interpreter()?, &script)?;

    Ok(parse_script_output(&output))
}

fn starts_with(entry: &DirEntry, pat: &str) -> bool {
    let name = entry.file_name();
    name.to_string_lossy().starts_with(pat)
}
fn ends_with(entry: &DirEntry, pat: &str) -> bool {
    let name = entry.file_name();
    name.to_string_lossy().ends_with(pat)
}

/// Finds the `_sysconfigdata*.py` file in the library path.
///
/// From the python source for `_sysconfigdata*.py` is always going to be located at
/// `build/lib.{PLATFORM}-{PY_MINOR_VERSION}` when built from source. The [exact line][1] is defined as:
///
/// ```py
/// pybuilddir = 'build/lib.%s-%s' % (get_platform(), sys.version_info[:2])
/// ```
///
/// Where get_platform returns a kebab-case formated string containing the os, the architecture and
/// possibly the os' kernel version (not the case on linux). However, when installed using a package
/// manager, the `_sysconfigdata*.py` file is installed in the `${PREFIX}/lib/python3.Y/` directory.
/// The `_sysconfigdata*.py` is generally in a sub-directory of the location of `libpython3.Y.so`.
/// So we must find the file in the following possible locations:
///
/// ```sh
/// # distribution from package manager, lib_dir should include lib/
/// ${INSTALL_PREFIX}/lib/python3.Y/_sysconfigdata*.py
/// ${INSTALL_PREFIX}/lib/libpython3.Y.so
/// ${INSTALL_PREFIX}/lib/python3.Y/config-3.Y-${HOST_TRIPLE}/libpython3.Y.so
///
/// # Built from source from host
/// ${CROSS_COMPILED_LOCATION}/build/lib.linux-x86_64-Y/_sysconfigdata*.py
/// ${CROSS_COMPILED_LOCATION}/libpython3.Y.so
///
/// # if cross compiled, kernel release is only present on certain OS targets.
/// ${CROSS_COMPILED_LOCATION}/build/lib.{OS}(-{OS-KERNEL-RELEASE})?-{ARCH}-Y/_sysconfigdata*.py
/// ${CROSS_COMPILED_LOCATION}/libpython3.Y.so
/// ```
///
/// [1]: https://github.com/python/cpython/blob/3.5/Lib/sysconfig.py#L389
fn find_sysconfigdata(cross: &CrossCompileConfig) -> Result<PathBuf> {
    let sysconfig_paths = search_lib_dir(&cross.lib_dir, &cross);
    let mut sysconfig_paths = sysconfig_paths
        .iter()
        .filter_map(|p| fs::canonicalize(p).ok())
        .collect::<Vec<PathBuf>>();
    sysconfig_paths.dedup();
    if sysconfig_paths.is_empty() {
        bail!(
            "Could not find either libpython.so or _sysconfigdata*.py in {}",
            cross.lib_dir.display()
        );
    } else if sysconfig_paths.len() > 1 {
        bail!(
            "Detected multiple possible python versions, please set the PYO3_PYTHON_VERSION \
            variable to the wanted version on your system\nsysconfigdata paths = {:?}",
            sysconfig_paths
        )
    }

    Ok(sysconfig_paths.remove(0))
}

/// recursive search for _sysconfigdata, returns all possibilities of sysconfigdata paths
fn search_lib_dir(path: impl AsRef<Path>, cross: &CrossCompileConfig) -> Vec<PathBuf> {
    let mut sysconfig_paths = vec![];
    let version_pat = if let Some(ref v) = cross.version {
        format!("python{}", v)
    } else {
        "python3.".into()
    };
    for f in fs::read_dir(path).expect("Path does not exist") {
        let sysc = match f {
            Ok(ref f) if starts_with(f, "_sysconfigdata") && ends_with(f, "py") => vec![f.path()],
            Ok(ref f) if starts_with(f, "build") => search_lib_dir(f.path(), cross),
            Ok(ref f) if starts_with(f, "lib.") => {
                let name = f.file_name();
                // check if right target os
                if !name.to_string_lossy().contains(if cross.os == "android" {
                    "linux"
                } else {
                    &cross.os
                }) {
                    continue;
                }
                // Check if right arch
                if !name.to_string_lossy().contains(&cross.arch) {
                    continue;
                }
                search_lib_dir(f.path(), cross)
            }
            Ok(ref f) if starts_with(f, &version_pat) => search_lib_dir(f.path(), cross),
            _ => continue,
        };
        sysconfig_paths.extend(sysc);
    }
    sysconfig_paths
}

/// Find cross compilation information from sysconfigdata file
///
/// first find sysconfigdata file which follows the pattern [`_sysconfigdata_{abi}_{platform}_{multiarch}`][1]
/// on python 3.6 or greater. On python 3.5 it is simply `_sysconfigdata.py`.
///
/// [1]: https://github.com/python/cpython/blob/3.8/Lib/sysconfig.py#L348
fn load_cross_compile_from_sysconfigdata(
    python_paths: CrossCompileConfig,
) -> Result<(InterpreterConfig, HashMap<String, String>)> {
    let sysconfig_path = find_sysconfigdata(&python_paths)?;
    let config_map = parse_sysconfigdata(sysconfig_path)?;

    let shared = config_map.get_bool("Py_ENABLE_SHARED")?;
    let major = config_map.get_numeric("version_major")?;
    let minor = config_map.get_numeric("version_minor")?;
    let ld_version = match config_map.get("LDVERSION") {
        Some(s) => s.clone(),
        None => format!("{}.{}", major, minor),
    };
    let calcsize_pointer = config_map.get_numeric("SIZEOF_VOID_P").ok();

    let python_version = PythonVersion {
        major,
        minor: Some(minor),
        implementation: PythonInterpreterKind::CPython,
    };

    let interpreter_config = InterpreterConfig {
        version: python_version,
        libdir: python_paths.lib_dir.to_str().map(String::from),
        shared,
        ld_version,
        base_prefix: "".to_string(),
        executable: PathBuf::new(),
        calcsize_pointer,
    };

    Ok((interpreter_config, fix_config_map(config_map)))
}

fn load_cross_compile_from_headers(
    python_paths: CrossCompileConfig,
) -> Result<(InterpreterConfig, HashMap<String, String>)> {
    let python_include_dir = python_paths.include_dir.unwrap();
    let python_include_dir = Path::new(&python_include_dir);
    let patchlevel_defines = parse_header_defines(python_include_dir.join("patchlevel.h"))?;

    let major = patchlevel_defines.get_numeric("PY_MAJOR_VERSION")?;
    let minor = patchlevel_defines.get_numeric("PY_MINOR_VERSION")?;

    let python_version = PythonVersion {
        major,
        minor: Some(minor),
        implementation: PythonInterpreterKind::CPython,
    };

    let config_map = parse_header_defines(python_include_dir.join("pyconfig.h"))?;
    let shared = config_map.get_bool("Py_ENABLE_SHARED")?;

    let interpreter_config = InterpreterConfig {
        version: python_version,
        libdir: python_paths.lib_dir.to_str().map(String::from),
        shared,
        ld_version: format!("{}.{}", major, minor),
        base_prefix: "".to_string(),
        executable: PathBuf::new(),
        calcsize_pointer: None,
    };

    Ok((interpreter_config, fix_config_map(config_map)))
}

fn load_cross_compile_info(
    python_paths: CrossCompileConfig,
) -> Result<(InterpreterConfig, HashMap<String, String>)> {
    let target_family = env::var("CARGO_CFG_TARGET_FAMILY")?;
    // Because compiling for windows on linux still includes the unix target family
    if target_family == "unix" {
        // Configure for unix platforms using the sysconfigdata file
        load_cross_compile_from_sysconfigdata(python_paths)
    } else {
        // Must configure by headers on windows platform
        load_cross_compile_from_headers(python_paths)
    }
}

/// Examine python's compile flags to pass to cfg by launching
/// the interpreter and printing variables of interest from
/// sysconfig.get_config_vars.
fn get_config_vars(python_path: &Path) -> Result<HashMap<String, String>> {
    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        return get_config_vars_windows(python_path);
    }

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

fn get_config_vars_windows(_: &Path) -> Result<HashMap<String, String>> {
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
fn run_python_script(interpreter: &Path, script: &str) -> Result<String> {
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
                    interpreter.display()
                );
            } else {
                bail!(
                    "Failed to run the Python interpreter at {}: {}",
                    interpreter.display(),
                    err
                );
            }
        }
        Ok(ref ok) if !ok.status.success() => bail!("Python script failed: {}"),
        Ok(ok) => Ok(String::from_utf8(ok.stdout)?),
    }
}

fn get_library_link_name(version: &PythonVersion, ld_version: &str) -> String {
    if cfg!(target_os = "windows") {
        // Mirrors the behavior in CPython's `PC/pyconfig.h`.
        if env::var_os("CARGO_FEATURE_ABI3").is_some() {
            return "python3".to_string();
        }

        let minor_or_empty_string = match version.minor {
            Some(minor) => format!("{}", minor),
            None => String::new(),
        };
        format!("python{}{}", version.major, minor_or_empty_string)
    } else {
        match version.implementation {
            PythonInterpreterKind::CPython => format!("python{}", ld_version),
            PythonInterpreterKind::PyPy => format!("pypy{}-c", version.major),
        }
    }
}

fn get_rustc_link_lib(config: &InterpreterConfig) -> Result<String> {
    match env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
        "windows" => get_rustc_link_lib_windows(config),
        "macos" => get_rustc_link_lib_macos(config),
        _ => get_rustc_link_lib_unix(config),
    }
}

fn get_rustc_link_lib_unix(config: &InterpreterConfig) -> Result<String> {
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

fn get_macos_linkmodel(config: &InterpreterConfig) -> Result<String> {
    // PyPy 3.6 ships with a shared library, but doesn't have Py_ENABLE_SHARED.
    if config.version.implementation == PythonInterpreterKind::PyPy {
        return Ok("shared".to_string());
    }

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

fn get_rustc_link_lib_macos(config: &InterpreterConfig) -> Result<String> {
    // os x can be linked to a framework or static or dynamic, and
    // Py_ENABLE_SHARED is wrong; framework means shared library
    let link_name = get_library_link_name(&config.version, &config.ld_version);
    match get_macos_linkmodel(config)?.as_ref() {
        "static" => Ok(format!("cargo:rustc-link-lib=static={}", link_name,)),
        "shared" => Ok(format!("cargo:rustc-link-lib={}", link_name)),
        "framework" => Ok(format!("cargo:rustc-link-lib={}", link_name,)),
        other => bail!("unknown linkmodel {}", other),
    }
}

fn get_rustc_link_lib_windows(config: &InterpreterConfig) -> Result<String> {
    // Py_ENABLE_SHARED doesn't seem to be present on windows.
    Ok(format!(
        "cargo:rustc-link-lib=pythonXY:{}",
        get_library_link_name(&config.version, &config.ld_version)
    ))
}

fn find_interpreter() -> Result<PathBuf> {
    if let Some(exe) = env::var_os("PYO3_PYTHON") {
        Ok(exe.into())
    } else if let Some(exe) = env::var_os("PYTHON_SYS_EXECUTABLE") {
        // Backwards-compatible name for PYO3_PYTHON; this may be removed at some point in the future.
        Ok(exe.into())
    } else {
        ["python", "python3"]
            .iter()
            .find(|bin| {
                if let Ok(out) = Command::new(bin).arg("--version").output() {
                    // begin with `Python 3.X.X :: additional info`
                    out.stdout.starts_with(b"Python 3") || out.stderr.starts_with(b"Python 3")
                } else {
                    false
                }
            })
            .map(PathBuf::from)
            .ok_or_else(|| "Python 3.x interpreter not found".into())
    }
}

/// Locate a suitable python interpreter and extract config from it.
///
/// The following locations are checked in the order listed:
///
/// 1. If `PYO3_PYTHON` is set, this intepreter is used and an error is raised if the
/// version doesn't match.
/// 2. `python`
/// 3. `python{major version}`
/// 4. `python{major version}.{minor version}`
///
/// If none of the above works, an error is returned
fn find_interpreter_and_get_config() -> Result<(InterpreterConfig, HashMap<String, String>)> {
    let python_interpreter = find_interpreter()?;
    let interpreter_config = get_config_from_interpreter(&python_interpreter)?;
    if interpreter_config.version.major == 3 {
        return Ok((interpreter_config, get_config_vars(&python_interpreter)?));
    }

    Err("No Python interpreter found".into())
}

/// Extract compilation vars from the specified interpreter.
fn get_config_from_interpreter(interpreter: &Path) -> Result<InterpreterConfig> {
    let script = r#"
import platform
import struct
import sys
import sysconfig
import os.path

PYPY = platform.python_implementation() == "PyPy"

# Anaconda based python distributions have a static python executable, but include
# the shared library. Use the shared library for embedding to avoid rust trying to
# LTO the static library (and failing with newer gcc's, because it is old).
ANACONDA = os.path.exists(os.path.join(sys.prefix, 'conda-meta'))

try:
    base_prefix = sys.base_prefix
except AttributeError:
    base_prefix = sys.exec_prefix

libdir = sysconfig.get_config_var('LIBDIR')

print("version_major", sys.version_info[0])
print("version_minor", sys.version_info[1])
print("implementation", platform.python_implementation())
if libdir is not None:
    print("libdir", libdir)
print("ld_version", sysconfig.get_config_var('LDVERSION') or sysconfig.get_config_var('py_version_short'))
print("base_prefix", base_prefix)
print("shared", PYPY or ANACONDA or bool(sysconfig.get_config_var('Py_ENABLE_SHARED')))
print("executable", sys.executable)
print("calcsize_pointer", struct.calcsize("P"))
"#;
    let output = run_python_script(interpreter, script)?;
    let map: HashMap<String, String> = parse_script_output(&output);
    Ok(InterpreterConfig {
        version: PythonVersion {
            major: map["version_major"].parse()?,
            minor: Some(map["version_minor"].parse()?),
            implementation: map["implementation"].parse()?,
        },
        libdir: map.get("libdir").cloned(),
        shared: map["shared"] == "True",
        ld_version: map["ld_version"].clone(),
        base_prefix: map["base_prefix"].clone(),
        executable: map["executable"].clone().into(),
        calcsize_pointer: Some(map["calcsize_pointer"].parse()?),
    })
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
    let target_os = env::var_os("CARGO_CFG_TARGET_OS").unwrap();

    let is_extension_module = env::var_os("CARGO_FEATURE_EXTENSION_MODULE").is_some();
    if !is_extension_module || target_os == "windows" || target_os == "android" {
        println!("{}", get_rustc_link_lib(&interpreter_config)?);
        if let Some(libdir) = &interpreter_config.libdir {
            println!("cargo:rustc-link-search=native={}", libdir);
        } else if target_os == "windows" {
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
        // fail PYO3_PYTHON=python2 cargo ...
        bail!("Python 2 is not supported");
    }

    if env::var_os("CARGO_FEATURE_ABI3").is_some() {
        println!("cargo:rustc-cfg=Py_LIMITED_API");
    }

    if let Some(minor) = interpreter_config.version.minor {
        for i in 6..=minor {
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

fn main() -> Result<()> {
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
    //
    // Detecting if cross-compiling by checking if the target triple is different from the host
    // rustc's triple.
    let (interpreter_config, mut config_map) = if let Some(paths) = cross_compiling()? {
        load_cross_compile_info(paths)?
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

    for var in ["LIB", "LD_LIBRARY_PATH", "PYO3_PYTHON"].iter() {
        println!("cargo:rerun-if-env-changed={}", var);
    }

    if env::var_os("PYO3_PYTHON").is_none() {
        // When PYO3_PYTHON is not used, PYTHON_SYS_EXECUTABLE has the highest priority.
        // Let's watch it.
        println!("cargo:rerun-if-env-changed=PYTHON_SYS_EXECUTABLE");
        if env::var_os("PYTHON_SYS_EXECUTABLE").is_none() {
            // When PYTHON_SYS_EXECUTABLE is also not used, then we use PATH.
            // Let's watch this, too.
            println!("cargo:rerun-if-env-changed=PATH");
        }
    }

    Ok(())
}
