use std::{
    collections::{HashMap, HashSet},
    convert::AsRef,
    env,
    ffi::OsString,
    fmt::Display,
    fs::{self, DirEntry},
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    str::FromStr,
};

use crate::{
    bail, ensure,
    errors::{Context, Error, Result},
    warn,
};

/// Minimum Python version PyO3 supports.
const MINIMUM_SUPPORTED_VERSION: PythonVersion = PythonVersion { major: 3, minor: 6 };
/// Maximum Python version that can be used as minimum required Python version with abi3.
const ABI3_MAX_MINOR: u8 = 9;

/// Gets an environment variable owned by cargo.
///
/// Environment variables set by cargo are expected to be valid UTF8.
pub fn cargo_env_var(var: &str) -> Option<String> {
    env::var_os(var).map(|os_string| os_string.to_str().unwrap().into())
}

/// Gets an external environment variable, and registers the build script to rerun if
/// the variable changes.
pub fn env_var(var: &str) -> Option<OsString> {
    println!("cargo:rerun-if-env-changed={}", var);
    env::var_os(var)
}

/// Configuration needed by PyO3 to build for the correct Python implementation.
///
/// Usually this is queried directly from the Python interpreter, or overridden using the
/// `PYO3_CONFIG_FILE` environment variable.
///
/// When the `PYO3_NO_PYTHON` variable is set, or during cross compile situations, then alternative
/// strategies are used to populate this type.
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct InterpreterConfig {
    pub implementation: PythonImplementation,
    pub version: PythonVersion,
    pub shared: bool,
    pub abi3: bool,
    pub lib_name: Option<String>,
    pub lib_dir: Option<String>,
    pub executable: Option<String>,
    pub pointer_width: Option<u32>,
    pub build_flags: BuildFlags,
}

impl InterpreterConfig {
    #[doc(hidden)]
    pub fn emit_pyo3_cfgs(&self) {
        // This should have been checked during pyo3-build-config build time.
        assert!(self.version >= MINIMUM_SUPPORTED_VERSION);
        for i in MINIMUM_SUPPORTED_VERSION.minor..=self.version.minor {
            println!("cargo:rustc-cfg=Py_3_{}", i);
        }

        if self.abi3 {
            println!("cargo:rustc-cfg=Py_LIMITED_API");
        }

        if self.implementation.is_pypy() {
            println!("cargo:rustc-cfg=PyPy");
            if self.abi3 {
                warn!(
                    "PyPy does not yet support abi3 so the build artifacts will be version-specific. \
                    See https://foss.heptapod.net/pypy/pypy/-/issues/3397 for more information."
                )
            }
        };

        for flag in &self.build_flags.0 {
            println!("cargo:rustc-cfg=py_sys_config=\"{}\"", flag)
        }
    }

    #[doc(hidden)]
    pub fn from_interpreter(interpreter: impl AsRef<Path>) -> Result<Self> {
        const SCRIPT: &str = r#"
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

# Windows always uses shared linking
WINDOWS = hasattr(platform, "win32_ver")

# macOS framework packages use shared linking
FRAMEWORK = bool(get_config_var("PYTHONFRAMEWORK"))

# unix-style shared library enabled
SHARED = bool(get_config_var("Py_ENABLE_SHARED"))

print("implementation", platform.python_implementation())
print("version_major", sys.version_info[0])
print("version_minor", sys.version_info[1])
print("shared", PYPY or ANACONDA or WINDOWS or FRAMEWORK or SHARED)
print_if_set("ld_version", get_config_var("LDVERSION"))
print_if_set("libdir", get_config_var("LIBDIR"))
print_if_set("base_prefix", base_prefix)
print("executable", sys.executable)
print("calcsize_pointer", struct.calcsize("P"))
"#;
        let output = run_python_script(interpreter.as_ref(), SCRIPT)?;
        let map: HashMap<String, String> = parse_script_output(&output);
        let shared = map["shared"].as_str() == "True";

        let version = PythonVersion {
            major: map["version_major"]
                .parse()
                .context("failed to parse major version")?,
            minor: map["version_minor"]
                .parse()
                .context("failed to parse minor version")?,
        };

        let abi3 = is_abi3();
        let implementation = map["implementation"].parse()?;

        let lib_name = if cfg!(windows) {
            default_lib_name_windows(
                version,
                abi3,
                &cargo_env_var("CARGO_CFG_TARGET_ENV").unwrap(),
            )
        } else {
            default_lib_name_unix(
                version,
                implementation,
                map.get("ld_version").map(String::as_str),
            )
        };

        let lib_dir = if cfg!(windows) {
            map.get("base_prefix")
                .map(|base_prefix| format!("{}\\libs", base_prefix))
        } else {
            map.get("libdir").cloned()
        };

        // The reason we don't use platform.architecture() here is that it's not
        // reliable on macOS. See https://stackoverflow.com/a/1405971/823869.
        // Similarly, sys.maxsize is not reliable on Windows. See
        // https://stackoverflow.com/questions/1405913/how-do-i-determine-if-my-python-shell-is-executing-in-32bit-or-64bit-mode-on-os/1405971#comment6209952_1405971
        // and https://stackoverflow.com/a/3411134/823869.
        let calcsize_pointer: u32 = map["calcsize_pointer"]
            .parse()
            .context("failed to parse calcsize_pointer")?;

        Ok(InterpreterConfig {
            version,
            implementation,
            shared,
            abi3,
            lib_name: Some(lib_name),
            lib_dir,
            executable: map.get("executable").cloned(),
            pointer_width: Some(calcsize_pointer * 8),
            build_flags: BuildFlags::from_interpreter(interpreter)?.fixup(version, implementation),
        })
    }

    #[doc(hidden)]
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let config_file = std::fs::File::open(path)
            .with_context(|| format!("failed to open PyO3 config file at {}", path.display()))?;
        let reader = std::io::BufReader::new(config_file);
        InterpreterConfig::from_reader(reader)
    }

    #[doc(hidden)]
    pub fn from_reader(reader: impl Read) -> Result<Self> {
        let reader = BufReader::new(reader);
        let lines = reader.lines();

        macro_rules! parse_value {
            ($variable:ident, $value:ident) => {
                $variable = Some($value.parse().context(format!(
                    concat!(
                        "failed to parse ",
                        stringify!($variable),
                        " from config value '{}'"
                    ),
                    $value
                ))?)
            };
        }

        let mut implementation = None;
        let mut version = None;
        let mut shared = None;
        let mut abi3 = None;
        let mut lib_name = None;
        let mut lib_dir = None;
        let mut executable = None;
        let mut pointer_width = None;
        let mut build_flags = None;

        for (i, line) in lines.enumerate() {
            let line = line.context("failed to read line from config")?;
            let mut split = line.splitn(2, '=');
            let (key, value) = (
                split
                    .next()
                    .expect("first splitn value should always be present"),
                split
                    .next()
                    .ok_or_else(|| format!("expected key=value pair on line {}", i + 1))?,
            );
            match key {
                "implementation" => parse_value!(implementation, value),
                "version" => parse_value!(version, value),
                "shared" => parse_value!(shared, value),
                "abi3" => parse_value!(abi3, value),
                "lib_name" => parse_value!(lib_name, value),
                "lib_dir" => parse_value!(lib_dir, value),
                "executable" => parse_value!(executable, value),
                "pointer_width" => parse_value!(pointer_width, value),
                "build_flags" => parse_value!(build_flags, value),
                unknown => bail!("unknown config key `{}`", unknown),
            }
        }

        let version = version.ok_or("missing value for version")?;
        let implementation = implementation.unwrap_or(PythonImplementation::CPython);
        let abi3 = abi3.unwrap_or(false);

        Ok(InterpreterConfig {
            implementation,
            version,
            shared: shared.unwrap_or(true),
            abi3,
            lib_name,
            lib_dir,
            executable,
            pointer_width,
            build_flags: build_flags.unwrap_or_else(|| {
                if abi3 {
                    BuildFlags::abi3()
                } else {
                    BuildFlags::default()
                }
                .fixup(version, implementation)
            }),
        })
    }

    #[doc(hidden)]
    pub fn to_writer(&self, mut writer: impl Write) -> Result<()> {
        macro_rules! write_line {
            ($value:ident) => {
                writeln!(writer, "{}={}", stringify!($value), self.$value).context(concat!(
                    "failed to write ",
                    stringify!($value),
                    " to config"
                ))
            };
        }

        macro_rules! write_option_line {
            ($value:ident) => {
                if let Some(value) = &self.$value {
                    writeln!(writer, "{}={}", stringify!($value), value).context(concat!(
                        "failed to write ",
                        stringify!($value),
                        " to config"
                    ))
                } else {
                    Ok(())
                }
            };
        }

        write_line!(implementation)?;
        write_line!(version)?;
        write_line!(shared)?;
        write_line!(abi3)?;
        write_option_line!(lib_name)?;
        write_option_line!(lib_dir)?;
        write_option_line!(executable)?;
        write_option_line!(pointer_width)?;
        write_line!(build_flags)?;
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PythonVersion {
    pub major: u8,
    pub minor: u8,
}

impl PythonVersion {
    const PY37: Self = PythonVersion { major: 3, minor: 7 };
}

impl Display for PythonVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl FromStr for PythonVersion {
    type Err = crate::errors::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut split = value.splitn(2, '.');
        let (major, minor) = (
            split
                .next()
                .expect("first splitn value should always be present"),
            split.next().ok_or("expected major.minor version")?,
        );
        Ok(Self {
            major: major.parse().context("failed to parse major version")?,
            minor: minor.parse().context("failed to parse minor version")?,
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PythonImplementation {
    CPython,
    PyPy,
}

impl PythonImplementation {
    #[doc(hidden)]
    pub fn is_pypy(self) -> bool {
        self == PythonImplementation::PyPy
    }
}

impl Display for PythonImplementation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PythonImplementation::CPython => write!(f, "CPython"),
            PythonImplementation::PyPy => write!(f, "PyPy"),
        }
    }
}

impl FromStr for PythonImplementation {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "CPython" => Ok(PythonImplementation::CPython),
            "PyPy" => Ok(PythonImplementation::PyPy),
            _ => bail!("unknown interpreter: {}", s),
        }
    }
}

fn is_abi3() -> bool {
    cargo_env_var("CARGO_FEATURE_ABI3").is_some()
}

trait GetPrimitive {
    fn get_bool(&self, key: &str) -> Result<bool>;
    fn get_numeric<T: FromStr>(&self, key: &str) -> Result<T>
    where
        T::Err: std::error::Error + 'static;
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

    fn get_numeric<T: FromStr>(&self, key: &str) -> Result<T>
    where
        T::Err: std::error::Error + 'static,
    {
        self.get(key)
            .ok_or(format!("{} is not defined", key))?
            .parse::<T>()
            .with_context(|| format!("Could not parse value of {}", key))
    }
}

struct CrossCompileConfig {
    lib_dir: PathBuf,
    version: Option<PythonVersion>,
    os: String,
    arch: String,
}

pub fn any_cross_compiling_env_vars_set() -> bool {
    env::var_os("PYO3_CROSS").is_some()
        || env::var_os("PYO3_CROSS_LIB_DIR").is_some()
        || env::var_os("PYO3_CROSS_PYTHON_VERSION").is_some()
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
                let utf8_str = os_string
                    .to_str()
                    .ok_or("PYO3_CROSS_PYTHON_VERSION is not valid utf-8.")?;
                utf8_str
                    .parse()
                    .context("failed to parse PYO3_CROSS_PYTHON_VERSION")
            })
            .transpose()?,
    }))
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum BuildFlag {
    WITH_THREAD,
    Py_DEBUG,
    Py_REF_DEBUG,
    Py_TRACE_REFS,
    COUNT_ALLOCS,
    Other(String),
}

impl Display for BuildFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildFlag::Other(flag) => write!(f, "{}", flag),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl FromStr for BuildFlag {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "WITH_THREAD" => Ok(BuildFlag::WITH_THREAD),
            "Py_DEBUG" => Ok(BuildFlag::Py_DEBUG),
            "Py_REF_DEBUG" => Ok(BuildFlag::Py_REF_DEBUG),
            "Py_TRACE_REFS" => Ok(BuildFlag::Py_TRACE_REFS),
            "COUNT_ALLOCS" => Ok(BuildFlag::COUNT_ALLOCS),
            other => Ok(BuildFlag::Other(other.to_owned())),
        }
    }
}

/// A list of python interpreter compile-time preprocessor defines that
/// we will pick up and pass to rustc via `--cfg=py_sys_config={varname}`;
/// this allows using them conditional cfg attributes in the .rs files, so
///
/// ```rust
/// #[cfg(py_sys_config="{varname}")]
/// # struct Foo;
/// ```
///
/// is the equivalent of `#ifdef {varname}` in C.
///
/// see Misc/SpecialBuilds.txt in the python source for what these mean.
#[cfg_attr(test, derive(Debug, PartialEq))]
#[derive(Default)]
pub struct BuildFlags(pub HashSet<BuildFlag>);

impl BuildFlags {
    const ALL: [BuildFlag; 5] = [
        // TODO: Remove WITH_THREAD once Python 3.6 support dropped (as it's always on).
        BuildFlag::WITH_THREAD,
        BuildFlag::Py_DEBUG,
        BuildFlag::Py_REF_DEBUG,
        BuildFlag::Py_TRACE_REFS,
        BuildFlag::COUNT_ALLOCS,
    ];

    pub fn new() -> Self {
        BuildFlags(HashSet::new())
    }

    fn from_config_map(config_map: &HashMap<String, String>) -> Self {
        Self(
            BuildFlags::ALL
                .iter()
                .cloned()
                .filter(|flag| {
                    config_map
                        .get(&flag.to_string())
                        .map_or(false, |value| value == "1")
                })
                .collect(),
        )
    }

    /// Examine python's compile flags to pass to cfg by launching
    /// the interpreter and printing variables of interest from
    /// sysconfig.get_config_vars.
    fn from_interpreter(interpreter: impl AsRef<Path>) -> Result<Self> {
        // If we're on a Windows host, then Python won't have any useful config vars
        if cfg!(windows) {
            return Ok(Self::windows_hardcoded());
        }

        let mut script = String::from("import sysconfig\n");
        script.push_str("config = sysconfig.get_config_vars()\n");

        for k in BuildFlags::ALL.iter() {
            script.push_str(&format!("print(config.get('{}', '0'))\n", k));
        }

        let stdout = run_python_script(interpreter.as_ref(), &script)?;
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
            .map(|(flag, _)| flag.clone())
            .collect();

        Ok(Self(flags))
    }

    fn windows_hardcoded() -> Self {
        // sysconfig is missing all the flags on windows, so we can't actually
        // query the interpreter directly for its build flags.
        let mut flags = HashSet::new();
        flags.insert(BuildFlag::WITH_THREAD);
        Self(flags)
    }

    pub fn abi3() -> Self {
        let mut flags = HashSet::new();
        flags.insert(BuildFlag::WITH_THREAD);
        Self(flags)
    }

    fn fixup(mut self, version: PythonVersion, implementation: PythonImplementation) -> Self {
        if self.0.contains(&BuildFlag::Py_DEBUG) {
            self.0.insert(BuildFlag::Py_REF_DEBUG);
            if version <= PythonVersion::PY37 {
                // Py_DEBUG only implies Py_TRACE_REFS until Python 3.7
                self.0.insert(BuildFlag::Py_TRACE_REFS);
            }
        }

        // WITH_THREAD is always on for Python 3.7, and for PyPy.
        if implementation == PythonImplementation::PyPy || version >= PythonVersion::PY37 {
            self.0.insert(BuildFlag::WITH_THREAD);
        }

        self
    }
}

impl Display for BuildFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for flag in &self.0 {
            if !first {
                write!(f, ",")?;
            } else {
                first = false;
            }
            write!(f, "{}", flag)?;
        }
        Ok(())
    }
}

impl FromStr for BuildFlags {
    type Err = std::convert::Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut flags = HashSet::new();
        for flag in value.split(',') {
            flags.insert(flag.parse().unwrap());
        }
        Ok(BuildFlags(flags))
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
    let mut script = fs::read_to_string(config_path.as_ref()).with_context(|| {
        format!(
            "failed to read config from {}",
            config_path.as_ref().display()
        )
    })?;
    script += r#"
print("version_major", build_time_vars["VERSION"][0])  # 3
print("version_minor", build_time_vars["VERSION"][2])  # E.g., 8
print("SOABI", build_time_vars.get("SOABI", ""))
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
/// Where get_platform returns a kebab-case formatted string containing the os, the architecture and
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
    let sysconfig_paths = search_lib_dir(&cross.lib_dir, cross);
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
        sysconfig_paths.extend(match &f {
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
        });
    }
    // If we got more than one file, only take those that contain the arch name.
    // For ubuntu 20.04 with host architecture x86_64 and a foreign architecture of armhf
    // this reduces the number of candidates to 1:
    //
    // $ find /usr/lib/python3.8/ -name '_sysconfigdata*.py' -not -lname '*'
    //  /usr/lib/python3.8/_sysconfigdata__x86_64-linux-gnu.py
    //  /usr/lib/python3.8/_sysconfigdata__arm-linux-gnueabihf.py
    if sysconfig_paths.len() > 1 {
        let temp = sysconfig_paths
            .iter()
            .filter(|p| p.to_string_lossy().contains(&cross.arch))
            .cloned()
            .collect::<Vec<PathBuf>>();
        if !temp.is_empty() {
            sysconfig_paths = temp;
        }
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
    let pointer_width = sysconfig_data
        .get_numeric("SIZEOF_VOID_P")
        .map(|bytes_width: u32| bytes_width * 8)
        .ok();
    let soabi = match sysconfig_data.get("SOABI") {
        Some(s) => s,
        None => bail!("sysconfigdata did not define SOABI"),
    };

    let version = PythonVersion { major, minor };
    let implementation = if soabi.starts_with("pypy") {
        PythonImplementation::PyPy
    } else if soabi.starts_with("cpython") {
        PythonImplementation::CPython
    } else {
        bail!("unsupported Python interpreter");
    };

    Ok(InterpreterConfig {
        implementation,
        version,
        shared: sysconfig_data.get_bool("Py_ENABLE_SHARED")?,
        abi3: is_abi3(),
        lib_dir: cross_compile_config.lib_dir.to_str().map(String::from),
        lib_name: Some(default_lib_name_unix(
            version,
            implementation,
            sysconfig_data.get("LDVERSION").map(String::as_str),
        )),
        executable: None,
        pointer_width,
        build_flags: BuildFlags::from_config_map(&sysconfig_data).fixup(version, implementation),
    })
}

fn windows_hardcoded_cross_compile(
    cross_compile_config: CrossCompileConfig,
) -> Result<InterpreterConfig> {
    let version = cross_compile_config.version.or_else(get_abi3_version)
        .ok_or("PYO3_CROSS_PYTHON_VERSION or an abi3-py3* feature must be specified when cross-compiling for Windows.")?;

    Ok(InterpreterConfig {
        implementation: PythonImplementation::CPython,
        version,
        shared: true,
        abi3: is_abi3(),
        lib_name: Some(default_lib_name_windows(version, false, "msvc")),
        lib_dir: cross_compile_config.lib_dir.to_str().map(String::from),
        executable: None,
        pointer_width: None,
        build_flags: BuildFlags::windows_hardcoded(),
    })
}

fn load_cross_compile_config(
    cross_compile_config: CrossCompileConfig,
) -> Result<InterpreterConfig> {
    match cargo_env_var("CARGO_CFG_TARGET_FAMILY") {
        // Configure for unix platforms using the sysconfigdata file
        Some(os) if os == "unix" => load_cross_compile_from_sysconfigdata(cross_compile_config),
        // Use hardcoded interpreter config when targeting Windows
        Some(os) if os == "windows" => windows_hardcoded_cross_compile(cross_compile_config),
        // sysconfigdata works fine on wasm/wasi
        Some(os) if os == "wasm" => load_cross_compile_from_sysconfigdata(cross_compile_config),
        // Waiting for users to tell us what they expect on their target platform
        Some(os) => bail!(
            "Unknown target OS family for cross-compilation: {:?}.\n\
            \n\
            Please set the PYO3_CONFIG_FILE environment variable to a config suitable for your \
            target interpreter.",
            os
        ),
        // Unknown os family - try to do something useful
        None => load_cross_compile_from_sysconfigdata(cross_compile_config),
    }
}

// Link against python3.lib for the stable ABI on Windows.
// See https://www.python.org/dev/peps/pep-0384/#linkage
//
// This contains only the limited ABI symbols.
const WINDOWS_ABI3_LIB_NAME: &str = "python3";

fn default_lib_name_windows(version: PythonVersion, abi3: bool, target_env: &str) -> String {
    if abi3 {
        WINDOWS_ABI3_LIB_NAME.to_owned()
    } else if target_env == "gnu" {
        // https://packages.msys2.org/base/mingw-w64-python
        format!("python{}.{}", version.major, version.minor)
    } else {
        format!("python{}{}", version.major, version.minor)
    }
}

fn default_lib_name_unix(
    version: PythonVersion,
    implementation: PythonImplementation,
    ld_version: Option<&str>,
) -> String {
    match implementation {
        PythonImplementation::CPython => match ld_version {
            Some(ld_version) => format!("python{}", ld_version),
            None => format!("python{}.{}", version.major, version.minor),
        },
        PythonImplementation::PyPy => format!("pypy{}-c", version.major),
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
        Ok(ok) => Ok(String::from_utf8(ok.stdout)
            .context("failed to parse Python script output as utf-8")?),
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

/// Attempts to locate a python interpreter.
///
/// Locations are checked in the order listed:
///   1. If `PYO3_PYTHON` is set, this interpreter is used.
///   2. If in a virtualenv, that environment's interpreter is used.
///   3. `python`, if this is functional a Python 3.x interpreter
///   4. `python3`, as above
pub fn find_interpreter() -> Result<PathBuf> {
    if let Some(exe) = env_var("PYO3_PYTHON") {
        Ok(exe.into())
    } else if let Some(venv_path) = get_venv_path() {
        // Use cfg rather can CARGO_TARGET_OS because this affects how files are located on the
        // host OS
        if cfg!(windows) {
            Ok(venv_path.join("Scripts\\python"))
        } else {
            Ok(venv_path.join("bin/python"))
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

pub fn get_abi3_version() -> Option<PythonVersion> {
    let minor_version = (MINIMUM_SUPPORTED_VERSION.minor..=ABI3_MAX_MINOR)
        .find(|i| cargo_env_var(&format!("CARGO_FEATURE_ABI3_PY3{}", i)).is_some());
    minor_version.map(|minor| PythonVersion { major: 3, minor })
}

/// Lowers the configured version to the abi3 version, if set.
fn fixup_config_for_abi3(
    config: &mut InterpreterConfig,
    abi3_version: Option<PythonVersion>,
) -> Result<()> {
    if let Some(version) = abi3_version {
        ensure!(
            version <= config.version,
            "cannot set a minimum Python version {} higher than the interpreter version {} \
             (the minimum Python version is implied by the abi3-py3{} feature)",
            version,
            config.version,
            version.minor,
        );

        config.version = version;
    }
    Ok(())
}

/// Generates an interpreter config suitable for cross-compilation.
///
/// This must be called from PyO3's build script, because it relies on environment variables such as
/// CARGO_CFG_TARGET_OS which aren't available at any other time.
pub fn make_cross_compile_config() -> Result<Option<InterpreterConfig>> {
    let mut interpreter_config = if let Some(paths) = cross_compiling()? {
        load_cross_compile_config(paths)?
    } else {
        return Ok(None);
    };
    fixup_config_for_abi3(&mut interpreter_config, get_abi3_version())?;
    Ok(Some(interpreter_config))
}

/// Generates an interpreter config which will be hard-coded into the pyo3-build-config crate.
/// Only used by `pyo3-build-config` build script.
#[allow(dead_code)]
pub fn make_interpreter_config() -> Result<InterpreterConfig> {
    let mut interpreter_config = InterpreterConfig::from_interpreter(find_interpreter()?)?;
    fixup_config_for_abi3(&mut interpreter_config, get_abi3_version())?;
    Ok(interpreter_config)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_config_file_roundtrip() {
        let config = InterpreterConfig {
            abi3: true,
            build_flags: BuildFlags::abi3(),
            pointer_width: Some(32),
            executable: Some("executable".into()),
            implementation: PythonImplementation::CPython,
            lib_name: Some("lib_name".into()),
            lib_dir: Some("lib_dir".into()),
            shared: true,
            version: MINIMUM_SUPPORTED_VERSION,
        };
        let mut buf: Vec<u8> = Vec::new();
        config.to_writer(&mut buf).unwrap();

        assert_eq!(
            config,
            InterpreterConfig::from_reader(Cursor::new(buf)).unwrap()
        );

        // And some different options, for variety

        let config = InterpreterConfig {
            abi3: false,
            build_flags: {
                let mut flags = HashSet::new();
                flags.insert(BuildFlag::Py_DEBUG);
                flags.insert(BuildFlag::Other(String::from("Py_SOME_FLAG")));
                BuildFlags(flags)
            },
            pointer_width: None,
            executable: None,
            implementation: PythonImplementation::PyPy,
            lib_dir: None,
            lib_name: None,
            shared: true,
            version: PythonVersion {
                major: 3,
                minor: 10,
            },
        };
        let mut buf: Vec<u8> = Vec::new();
        config.to_writer(&mut buf).unwrap();

        assert_eq!(
            config,
            InterpreterConfig::from_reader(Cursor::new(buf)).unwrap()
        );
    }

    #[test]
    fn test_config_file_defaults() {
        // Only version is required
        assert_eq!(
            InterpreterConfig::from_reader(Cursor::new("version=3.6")).unwrap(),
            InterpreterConfig {
                version: PythonVersion { major: 3, minor: 6 },
                implementation: PythonImplementation::CPython,
                shared: true,
                abi3: false,
                lib_name: None,
                lib_dir: None,
                executable: None,
                pointer_width: None,
                build_flags: BuildFlags::default(),
            }
        )
    }

    #[test]
    fn build_flags_default() {
        assert_eq!(BuildFlags::default(), BuildFlags::new());
    }

    #[test]
    fn build_flags_from_config_map() {
        let mut config_map = HashMap::new();

        assert_eq!(BuildFlags::from_config_map(&config_map).0, HashSet::new());

        for flag in &BuildFlags::ALL {
            config_map.insert(flag.to_string(), "0".into());
        }

        assert_eq!(BuildFlags::from_config_map(&config_map).0, HashSet::new());

        let mut expected_flags = HashSet::new();
        for flag in &BuildFlags::ALL {
            config_map.insert(flag.to_string(), "1".into());
            expected_flags.insert(flag.clone());
        }

        assert_eq!(BuildFlags::from_config_map(&config_map).0, expected_flags);
    }

    #[test]
    fn build_flags_fixup_py36_debug() {
        let mut build_flags = BuildFlags::new();
        build_flags.0.insert(BuildFlag::Py_DEBUG);

        build_flags = build_flags.fixup(
            PythonVersion { major: 3, minor: 6 },
            PythonImplementation::CPython,
        );

        // On 3.6, Py_DEBUG implies Py_REF_DEBUG and Py_TRACE_REFS
        assert!(build_flags.0.contains(&BuildFlag::Py_REF_DEBUG));
        assert!(build_flags.0.contains(&BuildFlag::Py_TRACE_REFS));
    }

    #[test]
    fn build_flags_fixup_py37_debug() {
        let mut build_flags = BuildFlags::new();
        build_flags.0.insert(BuildFlag::Py_DEBUG);

        build_flags = build_flags.fixup(PythonVersion::PY37, PythonImplementation::CPython);

        // On 3.7, Py_DEBUG implies Py_REF_DEBUG
        assert!(build_flags.0.contains(&BuildFlag::Py_REF_DEBUG));

        // 3.7 always has WITH_THREAD
        assert!(build_flags.0.contains(&BuildFlag::WITH_THREAD));
    }

    #[test]
    fn build_flags_fixup_pypy() {
        let mut build_flags = BuildFlags::new();

        build_flags = build_flags.fixup(
            PythonVersion { major: 3, minor: 6 },
            PythonImplementation::PyPy,
        );

        // PyPy always has WITH_THREAD
        assert!(build_flags.0.contains(&BuildFlag::WITH_THREAD));
    }

    #[test]
    fn parse_script_output() {
        let output = "foo bar\nbar foobar\n\n";
        let map = super::parse_script_output(output);
        assert_eq!(map.len(), 2);
        assert_eq!(map["foo"], "bar");
        assert_eq!(map["bar"], "foobar");
    }

    #[test]
    fn config_from_interpreter() {
        // Smoke test to just see whether this works
        //
        // PyO3's CI is dependent on Python being installed, so this should be reliable.
        assert!(make_interpreter_config().is_ok())
    }

    #[test]
    fn windows_hardcoded_cross_compile() {
        let cross_config = CrossCompileConfig {
            lib_dir: "C:\\some\\path".into(),
            version: Some(PythonVersion { major: 3, minor: 6 }),
            os: "os".into(),
            arch: "arch".into(),
        };

        assert_eq!(
            super::windows_hardcoded_cross_compile(cross_config).unwrap(),
            InterpreterConfig {
                implementation: PythonImplementation::CPython,
                version: PythonVersion { major: 3, minor: 6 },
                shared: true,
                abi3: false,
                lib_name: Some("python36".into()),
                lib_dir: Some("C:\\some\\path".into()),
                executable: None,
                pointer_width: None,
                build_flags: BuildFlags::windows_hardcoded()
            }
        );
    }

    #[test]
    fn default_lib_name_windows() {
        assert_eq!(
            super::default_lib_name_windows(PythonVersion { major: 3, minor: 6 }, false, "mvsc"),
            "python36",
        );
        assert_eq!(
            super::default_lib_name_windows(PythonVersion { major: 3, minor: 6 }, true, "mvsc"),
            "python3",
        );
        assert_eq!(
            super::default_lib_name_windows(PythonVersion { major: 3, minor: 6 }, false, "gnu"),
            "python3.6",
        );
        assert_eq!(
            super::default_lib_name_windows(PythonVersion { major: 3, minor: 6 }, true, "gnu"),
            "python3",
        );
    }

    #[test]
    fn default_lib_name_unix() {
        use PythonImplementation::*;
        // Defaults to pythonX.Y for CPython
        assert_eq!(
            super::default_lib_name_unix(PythonVersion { major: 3, minor: 6 }, CPython, None),
            "python3.6",
        );
        assert_eq!(
            super::default_lib_name_unix(PythonVersion { major: 3, minor: 9 }, CPython, None),
            "python3.9",
        );
        // Can use ldversion to override for CPython
        assert_eq!(
            super::default_lib_name_unix(
                PythonVersion { major: 3, minor: 9 },
                CPython,
                Some("3.7md")
            ),
            "python3.7md",
        );

        // PyPy ignores ldversion
        assert_eq!(
            super::default_lib_name_unix(PythonVersion { major: 3, minor: 9 }, PyPy, Some("3.7md")),
            "pypy3-c",
        );
    }

    #[test]
    fn interpreter_version_reduced_to_abi3() {
        let mut config = InterpreterConfig {
            abi3: true,
            build_flags: BuildFlags::new(),
            pointer_width: None,
            executable: None,
            implementation: PythonImplementation::CPython,
            lib_dir: None,
            lib_name: None,
            shared: true,
            version: PythonVersion { major: 3, minor: 7 },
        };

        fixup_config_for_abi3(&mut config, Some(PythonVersion { major: 3, minor: 6 })).unwrap();
        assert_eq!(config.version, PythonVersion { major: 3, minor: 6 });
    }

    #[test]
    fn abi3_version_cannot_be_higher_than_interpreter() {
        let mut config = InterpreterConfig {
            abi3: true,
            build_flags: BuildFlags::new(),
            pointer_width: None,
            executable: None,
            implementation: PythonImplementation::CPython,
            lib_dir: None,
            lib_name: None,
            shared: true,
            version: PythonVersion { major: 3, minor: 6 },
        };

        assert!(
            fixup_config_for_abi3(&mut config, Some(PythonVersion { major: 3, minor: 7 }))
                .unwrap_err()
                .to_string()
                .contains("cannot set a minimum Python version 3.7 higher than the interpreter version 3.6")
        );
    }
}
