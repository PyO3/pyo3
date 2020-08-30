use std::{
    collections::{HashMap, HashSet},
    convert::AsRef,
    env,
    ffi::OsString,
    fmt::Display,
    fs::{self, DirEntry},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    str::FromStr,
};

/// Minimum Python version PyO3 supports.
const MINIMUM_SUPPORTED_VERSION: PythonVersion = PythonVersion { major: 3, minor: 6 };
/// Maximum Python version that can be used as minimum required Python version with abi3.
const ABI3_MAX_MINOR: u8 = 9;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// A simple macro for returning an error. Resembles anyhow::bail.
macro_rules! bail {
    ($msg: expr) => { return Err($msg.into()); };
    ($fmt: literal $($args: tt)+) => { return Err(format!($fmt $($args)+).into()); };
}

// A simple macro for checking a condition. Resembles anyhow::ensure.
macro_rules! ensure {
    ($condition:expr, $($args: tt)+) => { if !($condition) { bail!($($args)+) } };
}

// Show warning. If needed, please extend this macro to support arguments.
macro_rules! warn {
    ($msg: literal) => {
        println!(concat!("cargo:warning=", $msg));
    };
}

/// Gets an environment variable owned by cargo.
///
/// Environment variables set by cargo are expected to be valid UTF8.
fn cargo_env_var(var: &str) -> Option<String> {
    env::var_os(var).map(|os_string| os_string.to_str().unwrap().into())
}

/// Gets an external environment variable, and registers the build script to rerun if
/// the variable changes.
fn env_var(var: &str) -> Option<OsString> {
    println!("cargo:rerun-if-env-changed={}", var);
    env::var_os(var)
}

/// Configuration needed by PyO3 to build for the correct Python implementation.
///
/// Usually this is queried directly from the Python interpreter. When the `PYO3_NO_PYTHON` variable
/// is set, or during cross compile situations, then alternative strategies are used to populate
/// this type.
#[derive(Debug)]
struct InterpreterConfig {
    version: PythonVersion,
    libdir: Option<String>,
    shared: bool,
    ld_version: Option<String>,
    base_prefix: Option<String>,
    executable: Option<String>,
    calcsize_pointer: Option<u32>,
    implementation: PythonImplementation,
    build_flags: BuildFlags,
}

impl InterpreterConfig {
    fn is_pypy(&self) -> bool {
        self.implementation == PythonImplementation::PyPy
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PythonVersion {
    major: u8,
    minor: u8,
}

impl PythonVersion {
    const PY37: Self = PythonVersion { major: 3, minor: 7 };
}

impl Display for PythonVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PythonImplementation {
    CPython,
    PyPy,
}

impl FromStr for PythonImplementation {
    type Err = Box<dyn std::error::Error>;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "CPython" => Ok(PythonImplementation::CPython),
            "PyPy" => Ok(PythonImplementation::PyPy),
            _ => bail!("Invalid interpreter: {}", s),
        }
    }
}

fn is_abi3() -> bool {
    cargo_env_var("CARGO_FEATURE_ABI3").is_some()
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
            _ => bail!("{} must be a bool (1/true/True or 0/false/False", key),
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
    version: Option<String>,
    os: String,
    arch: String,
}

fn cross_compiling() -> Result<Option<CrossCompileConfig>> {
    let cross = env_var("PYO3_CROSS");
    let cross_lib_dir = env_var("PYO3_CROSS_LIB_DIR");
    let cross_python_version = env_var("PYO3_CROSS_PYTHON_VERSION");

    let target_arch = cargo_env_var("CARGO_CFG_TARGET_ARCH");
    let target_vendor = cargo_env_var("CARGO_CFG_TARGET_VENDOR");
    let target_os = cargo_env_var("CARGO_CFG_TARGET_OS");

    if cross.is_none() && cross_lib_dir.is_none() && cross_python_version.is_none() {
        // No cross-compiling environment variables set; try to determine if this is a known case
        // which is not cross-compilation.

        let target = cargo_env_var("TARGET").unwrap();
        let host = cargo_env_var("HOST").unwrap();
        if target == host {
            // Not cross-compiling
            return Ok(None);
        }

        if target == "i686-pc-windows-msvc" && host == "x86_64-pc-windows-msvc" {
            // Not cross-compiling to compile for 32-bit Python from windows 64-bit
            return Ok(None);
        }

        if target == "x86_64-apple-darwin" && host == "aarch64-apple-darwin" {
            // Not cross-compiling to compile for x86-64 Python from macOS arm64
            return Ok(None);
        }

        if target == "aarch64-apple-darwin" && host == "x86_64-apple-darwin" {
            // Not cross-compiling to compile for arm64 Python from macOS x86_64
            return Ok(None);
        }

        if let (Some(arch), Some(vendor), Some(os)) = (&target_arch, &target_vendor, &target_os) {
            if host.starts_with(&format!("{}-{}-{}", arch, vendor, os)) {
                // Not cross-compiling if arch-vendor-os is all the same
                // e.g. x86_64-unknown-linux-musl on x86_64-unknown-linux-gnu host
                return Ok(None);
            }
        }
    }

    // At this point we assume that we are cross compiling.

    Ok(Some(CrossCompileConfig {
        lib_dir: cross_lib_dir
            .ok_or("The PYO3_CROSS_LIB_DIR environment variable must be set when cross-compiling")?
            .into(),
        os: target_os.unwrap(),
        arch: target_arch.unwrap(),
        version: cross_python_version
            .map(|os_string| {
                os_string
                    .to_str()
                    .ok_or("PYO3_CROSS_PYTHON_VERSION is not valid utf-8.")
                    .map(str::to_owned)
            })
            .transpose()?,
    }))
}

/// A list of python interpreter compile-time preprocessor defines that
/// we will pick up and pass to rustc via `--cfg=py_sys_config={varname}`;
/// this allows using them conditional cfg attributes in the .rs files, so
///
///     #[cfg(py_sys_config="{varname}"]
///
/// is the equivalent of `#ifdef {varname}` in C.
///
/// see Misc/SpecialBuilds.txt in the python source for what these mean.
#[derive(Debug)]
struct BuildFlags(HashSet<&'static str>);

impl BuildFlags {
    const ALL: [&'static str; 5] = [
        // TODO: Remove WITH_THREAD once Python 3.6 support dropped (as it's always on).
        "WITH_THREAD",
        "Py_DEBUG",
        "Py_REF_DEBUG",
        "Py_TRACE_REFS",
        "COUNT_ALLOCS",
    ];

    fn from_config_map(config_map: &HashMap<String, String>) -> Self {
        Self(
            BuildFlags::ALL
                .iter()
                .copied()
                .filter(|flag| config_map.get(*flag).map_or(false, |value| value == "1"))
                .collect(),
        )
    }

    /// Examine python's compile flags to pass to cfg by launching
    /// the interpreter and printing variables of interest from
    /// sysconfig.get_config_vars.
    fn from_interpreter(interpreter: &Path) -> Result<Self> {
        if cargo_env_var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
            return Ok(Self::windows_hardcoded());
        }

        let mut script = String::from("import sysconfig\n");
        script.push_str("config = sysconfig.get_config_vars()\n");

        for k in BuildFlags::ALL.iter() {
            script.push_str(&format!("print(config.get('{}', '0'))\n", k));
        }

        let stdout = run_python_script(&interpreter, &script)?;
        let split_stdout: Vec<&str> = stdout.trim_end().lines().collect();
        ensure!(
            split_stdout.len() == BuildFlags::ALL.len(),
            "Python stdout len didn't return expected number of lines: {}",
            split_stdout.len()
        );
        let flags = BuildFlags::ALL
            .iter()
            .zip(split_stdout)
            .filter(|(_, flag_value)| *flag_value == "1")
            .map(|(&flag, _)| flag)
            .collect();

        Ok(Self(flags))
    }

    fn windows_hardcoded() -> Self {
        // sysconfig is missing all the flags on windows, so we can't actually
        // query the interpreter directly for its build flags.
        let mut flags = HashSet::new();
        flags.insert("WITH_THREAD");

        // Uncomment these manually if your python was built with these and you want
        // the cfg flags to be set in rust.
        //
        // map.insert("Py_DEBUG", "1");
        // map.insert("Py_REF_DEBUG", "1");
        // map.insert("Py_TRACE_REFS", "1");
        // map.insert("COUNT_ALLOCS", 1");
        Self(flags)
    }

    fn abi3() -> Self {
        let mut flags = HashSet::new();
        flags.insert("WITH_THREAD");
        Self(flags)
    }

    fn fixup(mut self, version: PythonVersion, implementation: PythonImplementation) -> Self {
        if self.0.contains("Py_DEBUG") {
            self.0.insert("Py_REF_DEBUG");
            if version <= PythonVersion::PY37 {
                // Py_DEBUG only implies Py_TRACE_REFS until Python 3.7
                self.0.insert("Py_TRACE_REFS");
            }
        }

        // WITH_THREAD is always on for Python 3.7, and for PyPy.
        if implementation == PythonImplementation::PyPy || version >= PythonVersion::PY37 {
            self.0.insert("WITH_THREAD");
        }

        self
    }
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
KEYS = [
    "WITH_THREAD",
    "Py_DEBUG",
    "Py_REF_DEBUG",
    "Py_TRACE_REFS",
    "COUNT_ALLOCS",
    "Py_ENABLE_SHARED",
    "LDVERSION",
    "SIZEOF_VOID_P"
]
for key in KEYS:
    print(key, build_time_vars.get(key, 0))
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
    let sysconfig_name = env_var("_PYTHON_SYSCONFIGDATA_NAME");
    let mut sysconfig_paths = sysconfig_paths
        .iter()
        .filter_map(|p| {
            let canonical = fs::canonicalize(p).ok();
            match &sysconfig_name {
                Some(_) => canonical.filter(|p| p.file_stem() == sysconfig_name.as_deref()),
                None => canonical,
            }
        })
        .collect::<Vec<PathBuf>>();
    sysconfig_paths.dedup();
    if sysconfig_paths.is_empty() {
        bail!(
            "Could not find either libpython.so or _sysconfigdata*.py in {}",
            cross.lib_dir.display()
        );
    } else if sysconfig_paths.len() > 1 {
        let mut error_msg = String::from(
            "Detected multiple possible Python versions. Please set either the \
            PYO3_CROSS_PYTHON_VERSION variable to the wanted version or the \
            _PYTHON_SYSCONFIGDATA_NAME variable to the wanted sysconfigdata file name.\n\n\
            sysconfigdata files found:",
        );
        for path in sysconfig_paths {
            error_msg += &format!("\n\t{}", path.display());
        }
        bail!("{}", error_msg);
    }

    Ok(sysconfig_paths.remove(0))
}

/// recursive search for _sysconfigdata, returns all possibilities of sysconfigdata paths
fn search_lib_dir(path: impl AsRef<Path>, cross: &CrossCompileConfig) -> Vec<PathBuf> {
    let mut sysconfig_paths = vec![];
    let version_pat = if let Some(v) = &cross.version {
        format!("python{}", v)
    } else {
        "python3.".into()
    };
    for f in fs::read_dir(path).expect("Path does not exist") {
        let sysc = match &f {
            Ok(f) if starts_with(f, "_sysconfigdata") && ends_with(f, "py") => vec![f.path()],
            Ok(f) if starts_with(f, "build") => search_lib_dir(f.path(), cross),
            Ok(f) if starts_with(f, "lib.") => {
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
            Ok(f) if starts_with(f, &version_pat) => search_lib_dir(f.path(), cross),
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
    cross_compile_config: CrossCompileConfig,
) -> Result<InterpreterConfig> {
    let sysconfig_path = find_sysconfigdata(&cross_compile_config)?;
    let sysconfig_data = parse_sysconfigdata(sysconfig_path)?;

    let major = sysconfig_data.get_numeric("version_major")?;
    let minor = sysconfig_data.get_numeric("version_minor")?;
    let ld_version = match sysconfig_data.get("LDVERSION") {
        Some(s) => s.clone(),
        None => format!("{}.{}", major, minor),
    };
    let calcsize_pointer = sysconfig_data.get_numeric("SIZEOF_VOID_P").ok();

    let version = PythonVersion { major, minor };
    let implementation = PythonImplementation::CPython;

    Ok(InterpreterConfig {
        version,
        libdir: cross_compile_config.lib_dir.to_str().map(String::from),
        shared: sysconfig_data.get_bool("Py_ENABLE_SHARED")?,
        ld_version: Some(ld_version),
        base_prefix: None,
        executable: None,
        calcsize_pointer,
        implementation,
        build_flags: BuildFlags::from_config_map(&sysconfig_data).fixup(version, implementation),
    })
}

fn windows_hardcoded_cross_compile(
    cross_compile_config: CrossCompileConfig,
) -> Result<InterpreterConfig> {
    let (major, minor) = if let Some(version) = cross_compile_config.version {
        let mut parts = version.split('.');
        match (
            parts.next().and_then(|major| major.parse().ok()),
            parts.next().and_then(|minor| minor.parse().ok()),
            parts.next(),
        ) {
            (Some(major), Some(minor), None) => (major, minor),
            _ => bail!(
                "Expected major.minor version (e.g. 3.9) for PYO3_CROSS_PYTHON_VERSION, got `{}`",
                version
            ),
        }
    } else if let Some(minor_version) = get_abi3_minor_version() {
        (3, minor_version)
    } else {
        bail!("PYO3_CROSS_PYTHON_VERSION or an abi3-py3* feature must be specified when cross-compiling for Windows.")
    };

    Ok(InterpreterConfig {
        version: PythonVersion { major, minor },
        libdir: cross_compile_config.lib_dir.to_str().map(String::from),
        shared: true,
        ld_version: None,
        base_prefix: None,
        executable: None,
        calcsize_pointer: None,
        implementation: PythonImplementation::CPython,
        build_flags: BuildFlags::windows_hardcoded(),
    })
}

fn load_cross_compile_info(cross_compile_config: CrossCompileConfig) -> Result<InterpreterConfig> {
    match cargo_env_var("CARGO_CFG_TARGET_FAMILY") {
        // Configure for unix platforms using the sysconfigdata file
        Some(os) if os == "unix" => load_cross_compile_from_sysconfigdata(cross_compile_config),
        // Use hardcoded interpreter config when targeting Windows
        Some(os) if os == "windows" => windows_hardcoded_cross_compile(cross_compile_config),
        // Waiting for users to tell us what they expect on their target platform
        Some(os) => bail!(
            "Unsupported target OS family for cross-compilation: {:?}",
            os
        ),
        // Unknown os family - try to do something useful
        None => load_cross_compile_from_sysconfigdata(cross_compile_config),
    }
}

/// Run a python script using the specified interpreter binary.
fn run_python_script(interpreter: &Path, script: &str) -> Result<String> {
    let out = Command::new(interpreter)
        .env("PYTHONIOENCODING", "utf-8")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            child
                .stdin
                .as_mut()
                .expect("piped stdin")
                .write_all(script.as_bytes())?;
            child.wait_with_output()
        });

    match out {
        Err(err) => bail!(
            "failed to run the Python interpreter at {}: {}",
            interpreter.display(),
            err
        ),
        Ok(ok) if !ok.status.success() => bail!("Python script failed"),
        Ok(ok) => Ok(String::from_utf8(ok.stdout)?),
    }
}

fn get_library_link_name_unix(config: &InterpreterConfig) -> Result<String> {
    match config.implementation {
        PythonImplementation::CPython => match &config.ld_version {
            Some(ld_version) => Ok(format!("python{}", ld_version)),
            None => bail!("failed to configure `ld_version` when compiling for unix"),
        },
        PythonImplementation::PyPy => Ok(format!("pypy{}-c", config.version.major)),
    }
}

fn get_library_link_name(config: &InterpreterConfig) -> Result<String> {
    if cargo_env_var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        Ok(if is_abi3() {
            // Link against python3.lib for the stable ABI on Windows.
            // See https://www.python.org/dev/peps/pep-0384/#linkage
            //
            // This contains only the limited ABI symbols.
            "pythonXY:python3".to_owned()
        } else if cargo_env_var("CARGO_CFG_TARGET_ENV").unwrap() == "gnu" {
            // https://packages.msys2.org/base/mingw-w64-python
            format!(
                "pythonXY:python{}.{}",
                config.version.major, config.version.minor
            )
        } else {
            format!(
                "pythonXY:python{}{}",
                config.version.major, config.version.minor
            )
        })
    } else {
        get_library_link_name_unix(config)
    }
}

fn get_venv_path() -> Option<PathBuf> {
    match (env_var("VIRTUAL_ENV"), env_var("CONDA_PREFIX")) {
        (Some(dir), None) => Some(PathBuf::from(dir)),
        (None, Some(dir)) => Some(PathBuf::from(dir)),
        (Some(_), Some(_)) => {
            warn!(
                "Both VIRTUAL_ENV and CONDA_PREFIX are set. PyO3 will ignore both of these for \
                 locating the Python interpreter until you unset one of them."
            );
            None
        }
        (None, None) => None,
    }
}

/// Attempts to locate a python interpreter. Locations are checked in the order listed:
/// 1. If `PYO3_PYTHON` is set, this intepreter is used.
/// 2. If in a virtualenv, that environment's interpreter is used.
/// 3. `python`, if this is functional a Python 3.x interpreter
/// 4. `python3`, as above
fn find_interpreter() -> Result<PathBuf> {
    if let Some(exe) = env_var("PYO3_PYTHON") {
        Ok(exe.into())
    } else if let Some(venv_path) = get_venv_path() {
        match cargo_env_var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
            "windows" => Ok(venv_path.join("Scripts\\python")),
            _ => Ok(venv_path.join("bin/python")),
        }
    } else {
        println!("cargo:rerun-if-env-changed=PATH");
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
            .ok_or_else(|| "no Python 3.x interpreter found".into())
    }
}

/// Extract compilation vars from the specified interpreter.
fn get_config_from_interpreter(interpreter: &Path) -> Result<InterpreterConfig> {
    let script = r#"
# Allow the script to run on Python 2, so that nicer error can be printed later.
from __future__ import print_function

import os.path
import platform
import struct
import sys
from sysconfig import get_config_var

PYPY = platform.python_implementation() == "PyPy"

# sys.base_prefix is missing on Python versions older than 3.3; this allows the script to continue
# so that the version mismatch can be reported in a nicer way later.
base_prefix = getattr(sys, "base_prefix", None)

if base_prefix:
    # Anaconda based python distributions have a static python executable, but include
    # the shared library. Use the shared library for embedding to avoid rust trying to
    # LTO the static library (and failing with newer gcc's, because it is old).
    ANACONDA = os.path.exists(os.path.join(base_prefix, "conda-meta"))
else:
    ANACONDA = False

def print_if_set(varname, value):
    if value is not None:
        print(varname, value)

libdir = get_config_var("LIBDIR")

print("version_major", sys.version_info[0])
print("version_minor", sys.version_info[1])
print("implementation", platform.python_implementation())
print_if_set("libdir", libdir)
print_if_set("ld_version", get_config_var("LDVERSION"))
print_if_set("base_prefix", base_prefix)
print("framework", bool(get_config_var("PYTHONFRAMEWORK")))
print("shared", PYPY or ANACONDA or bool(get_config_var("Py_ENABLE_SHARED")))
print("executable", sys.executable)
print("calcsize_pointer", struct.calcsize("P"))
"#;
    let output = run_python_script(interpreter, script)?;
    let map: HashMap<String, String> = parse_script_output(&output);
    let shared = match (
        cargo_env_var("CARGO_CFG_TARGET_OS").unwrap().as_str(),
        map["framework"].as_str(),
        map["shared"].as_str(),
    ) {
        (_, _, "True")            // Py_ENABLE_SHARED is set
        | ("windows", _, _)       // Windows always uses shared linking
        | ("macos", "True", _)    // MacOS framework package uses shared linking
          => true,
        (_, _, "False") => false, // Any other platform, Py_ENABLE_SHARED not set
        _ => bail!("Unrecognised link model combination")
    };

    let version = PythonVersion {
        major: map["version_major"].parse()?,
        minor: map["version_minor"].parse()?,
    };

    let implementation = map["implementation"].parse()?;

    Ok(InterpreterConfig {
        version,
        implementation,
        libdir: map.get("libdir").cloned(),
        shared,
        ld_version: map.get("ld_version").cloned(),
        base_prefix: map.get("base_prefix").cloned(),
        executable: map.get("executable").cloned(),
        calcsize_pointer: Some(map["calcsize_pointer"].parse()?),
        build_flags: BuildFlags::from_interpreter(interpreter)?.fixup(version, implementation),
    })
}

fn ensure_python_version(interpreter_config: &InterpreterConfig) -> Result<()> {
    ensure!(
        interpreter_config.version >= MINIMUM_SUPPORTED_VERSION,
        "the configured Python interpreter version ({}) is lower than PyO3's minimum supported version ({})",
        interpreter_config.version,
        MINIMUM_SUPPORTED_VERSION,
    );

    Ok(())
}

fn emit_cargo_configuration(interpreter_config: &InterpreterConfig) -> Result<()> {
    let target_os = cargo_env_var("CARGO_CFG_TARGET_OS").unwrap();

    // Library search paths

    if let Some(libdir) = &interpreter_config.libdir {
        println!("cargo:rustc-link-search=native={}", libdir);
    }
    if target_os == "windows" {
        // libdir is only present on windows when cross-compiling, base_prefix is used on
        // windows hosts
        if let Some(base_prefix) = &interpreter_config.base_prefix {
            println!("cargo:rustc-link-search=native={}\\libs", base_prefix);
        }
    }

    if interpreter_config.implementation == PythonImplementation::PyPy {
        // PyPy 7.3.4 changed LIBDIR to point to base_prefix/lib as a regression, so need
        // to hard-code /bin search path too: https://foss.heptapod.net/pypy/pypy/-/issues/3442
        //
        // TODO: this workaround can probably be removed when PyPy 7.3.5 is released (and we
        // can call it a PyPy bug).
        if let Some(base_prefix) = &interpreter_config.base_prefix {
            println!("cargo:rustc-link-search=native={}/bin", base_prefix);
        }
    }

    // Link libpython (if appropriate)

    let is_extension_module = cargo_env_var("CARGO_FEATURE_EXTENSION_MODULE").is_some();
    match (is_extension_module, target_os.as_str()) {
        (_, "windows") | (_, "android") | (false, _) => {
            // windows or android - always link to libpython
            // other systems - only link libs if not extension module
            println!(
                "cargo:rustc-link-lib={link_model}{link_name}",
                link_model = if interpreter_config.shared {
                    ""
                } else {
                    "static="
                },
                link_name = get_library_link_name(&interpreter_config)?
            );
        }
        (true, _) => {
            // Extension module on unix system - only link non-lib targets
            if target_os == "macos" {
                // with extension module on macos some extra linker arguments are needed
                println!("cargo:rustc-cdylib-link-arg=-undefined");
                println!("cargo:rustc-cdylib-link-arg=dynamic_lookup");
            }

            let lib_name = get_library_link_name_unix(&interpreter_config)?;
            println!("cargo:rustc-link-arg-bins=-l{}", lib_name);
            println!("cargo:rustc-link-arg-tests=-l{}", lib_name);
            println!("cargo:rustc-link-arg-benches=-l{}", lib_name);
            println!("cargo:rustc-link-arg-examples=-l{}", lib_name);
        }
    }

    // Sanity checks to prevent newcomers attempting static embedding

    if env::var_os("CARGO_FEATURE_AUTO_INITIALIZE").is_some() {
        ensure!(
            interpreter_config.shared,
            "The `auto-initialize` feature is enabled, but your python installation only supports \
            embedding the Python interpreter statically. If you are attempting to run tests, or a \
            binary which is okay to link dynamically, install a Python distribution which ships \
            with the Python shared library.\n\
            \n\
            Embedding the Python interpreter statically does not yet have first-class support in \
            PyO3. If you are sure you intend to do this, disable the `auto-initialize` feature.\n\
            \n\
            For more information, see \
            https://pyo3.rs/v{pyo3_version}/\
                building_and_distribution.html#embedding-python-in-rust",
            pyo3_version = env::var("CARGO_PKG_VERSION").unwrap()
        );

        // TODO: PYO3_CI env is a hack to workaround CI with PyPy, where the `dev-dependencies`
        // currently cause `auto-initialize` to be enabled in CI.
        // Once cargo's `resolver = "2"` is stable (~ MSRV Rust 1.52), remove the check on the env var.
        ensure!(
            !interpreter_config.is_pypy() || env::var_os("PYO3_CI").is_some(),
            "The `auto-initialize` feature is not supported with PyPy."
        );
    }

    // Py_LIMITED_API cfg

    let is_abi3 = is_abi3();

    if interpreter_config.is_pypy() {
        println!("cargo:rustc-cfg=PyPy");
        if is_abi3 {
            warn!(
                "PyPy does not yet support abi3 so the build artifacts will be version-specific. \
                See https://foss.heptapod.net/pypy/pypy/-/issues/3397 for more information."
            )
        }
    };

    let minor = if is_abi3 {
        println!("cargo:rustc-cfg=Py_LIMITED_API");
        // Check any `abi3-py3*` feature is set. If not, use the interpreter version.
        match get_abi3_minor_version() {
            Some(minor) if minor > interpreter_config.version.minor => bail!(
                "You cannot set a mininimum Python version 3.{} higher than the interpreter version 3.{}",
                minor,
                interpreter_config.version.minor
            ),
            Some(minor) => minor,
            None => interpreter_config.version.minor
        }
    } else {
        interpreter_config.version.minor
    };

    // Py_3_x cfgs

    for i in MINIMUM_SUPPORTED_VERSION.minor..=minor {
        println!("cargo:rustc-cfg=Py_3_{}", i);
    }

    // py_sys_config cfgs

    for flag in &interpreter_config.build_flags.0 {
        println!("cargo:rustc-cfg=py_sys_config=\"{}\"", flag)
    }

    Ok(())
}

fn ensure_target_architecture(interpreter_config: &InterpreterConfig) -> Result<()> {
    // Try to check whether the target architecture matches the python library
    let rust_target = match cargo_env_var("CARGO_CFG_TARGET_POINTER_WIDTH")
        .unwrap()
        .as_str()
    {
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

    ensure!(
        rust_target == python_target,
        "Your Rust target architecture ({}) does not match your python interpreter ({})",
        rust_target,
        python_target
    );

    Ok(())
}

fn get_abi3_minor_version() -> Option<u8> {
    (MINIMUM_SUPPORTED_VERSION.minor..=ABI3_MAX_MINOR)
        .find(|i| cargo_env_var(&format!("CARGO_FEATURE_ABI3_PY3{}", i)).is_some())
}

fn get_interpreter_config() -> Result<InterpreterConfig> {
    // If PYO3_NO_PYTHON is set with abi3, we can build PyO3 without calling Python.
    if let Some(abi3_version) = get_abi3_minor_version() {
        if env_var("PYO3_NO_PYTHON").is_some() {
            return Ok(InterpreterConfig {
                version: PythonVersion {
                    major: 3,
                    minor: abi3_version,
                },
                implementation: PythonImplementation::CPython,
                libdir: None,
                build_flags: BuildFlags::abi3(),
                base_prefix: None,
                calcsize_pointer: None,
                executable: None,
                ld_version: None,
                shared: true,
            });
        }
    }

    if let Some(paths) = cross_compiling()? {
        load_cross_compile_info(paths)
    } else {
        get_config_from_interpreter(&find_interpreter()?)
    }
}

fn configure_pyo3() -> Result<()> {
    let interpreter_config = get_interpreter_config()?;
    ensure_python_version(&interpreter_config)?;
    ensure_target_architecture(&interpreter_config)?;
    emit_cargo_configuration(&interpreter_config)
}

fn main() {
    // Print out error messages using display, to get nicer formatting.
    if let Err(e) = configure_pyo3() {
        eprintln!("error: {}", e);
        std::process::exit(1)
    }
}
