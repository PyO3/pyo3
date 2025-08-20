//! Main implementation module included in both the `pyo3-build-config` library crate
//! and its build script.

// Optional python3.dll import library generator for Windows
#[cfg(feature = "python3-dll-a")]
#[path = "import_lib.rs"]
mod import_lib;

#[cfg(test)]
use std::cell::RefCell;
use std::{
    collections::{HashMap, HashSet},
    env,
    ffi::{OsStr, OsString},
    fmt::Display,
    fs::{self, DirEntry},
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    str::{self, FromStr},
};

pub use target_lexicon::Triple;

use target_lexicon::{Architecture, Environment, OperatingSystem, Vendor};

use crate::{
    bail, ensure,
    errors::{Context, Error, Result},
    warn,
};

/// Minimum Python version PyO3 supports.
pub(crate) const MINIMUM_SUPPORTED_VERSION: PythonVersion = PythonVersion { major: 3, minor: 7 };

/// GraalPy may implement the same CPython version over multiple releases.
const MINIMUM_SUPPORTED_VERSION_GRAALPY: PythonVersion = PythonVersion {
    major: 24,
    minor: 0,
};

/// Maximum Python version that can be used as minimum required Python version with abi3.
pub(crate) const ABI3_MAX_MINOR: u8 = 14;

#[cfg(test)]
thread_local! {
    static READ_ENV_VARS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// Gets an environment variable owned by cargo.
///
/// Environment variables set by cargo are expected to be valid UTF8.
pub fn cargo_env_var(var: &str) -> Option<String> {
    env::var_os(var).map(|os_string| os_string.to_str().unwrap().into())
}

/// Gets an external environment variable, and registers the build script to rerun if
/// the variable changes.
pub fn env_var(var: &str) -> Option<OsString> {
    if cfg!(feature = "resolve-config") {
        println!("cargo:rerun-if-env-changed={var}");
    }
    #[cfg(test)]
    {
        READ_ENV_VARS.with(|env_vars| {
            env_vars.borrow_mut().push(var.to_owned());
        });
    }
    env::var_os(var)
}

/// Gets the compilation target triple from environment variables set by Cargo.
///
/// Must be called from a crate build script.
pub fn target_triple_from_env() -> Triple {
    env::var("TARGET")
        .expect("target_triple_from_env() must be called from a build script")
        .parse()
        .expect("Unrecognized TARGET environment variable value")
}

/// Configuration needed by PyO3 to build for the correct Python implementation.
///
/// Usually this is queried directly from the Python interpreter, or overridden using the
/// `PYO3_CONFIG_FILE` environment variable.
///
/// When the `PYO3_NO_PYTHON` variable is set, or during cross compile situations, then alternative
/// strategies are used to populate this type.
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct InterpreterConfig {
    /// The Python implementation flavor.
    ///
    /// Serialized to `implementation`.
    pub implementation: PythonImplementation,

    /// Python `X.Y` version. e.g. `3.9`.
    ///
    /// Serialized to `version`.
    pub version: PythonVersion,

    /// Whether link library is shared.
    ///
    /// Serialized to `shared`.
    pub shared: bool,

    /// Whether linking against the stable/limited Python 3 API.
    ///
    /// Serialized to `abi3`.
    pub abi3: bool,

    /// The name of the link library defining Python.
    ///
    /// This effectively controls the `cargo:rustc-link-lib=<name>` value to
    /// control how libpython is linked. Values should not contain the `lib`
    /// prefix.
    ///
    /// Serialized to `lib_name`.
    pub lib_name: Option<String>,

    /// The directory containing the Python library to link against.
    ///
    /// The effectively controls the `cargo:rustc-link-search=native=<path>` value
    /// to add an additional library search path for the linker.
    ///
    /// Serialized to `lib_dir`.
    pub lib_dir: Option<String>,

    /// Path of host `python` executable.
    ///
    /// This is a valid executable capable of running on the host/building machine.
    /// For configurations derived by invoking a Python interpreter, it was the
    /// executable invoked.
    ///
    /// Serialized to `executable`.
    pub executable: Option<String>,

    /// Width in bits of pointers on the target machine.
    ///
    /// Serialized to `pointer_width`.
    pub pointer_width: Option<u32>,

    /// Additional relevant Python build flags / configuration settings.
    ///
    /// Serialized to `build_flags`.
    pub build_flags: BuildFlags,

    /// Whether to suppress emitting of `cargo:rustc-link-*` lines from the build script.
    ///
    /// Typically, `pyo3`'s build script will emit `cargo:rustc-link-lib=` and
    /// `cargo:rustc-link-search=` lines derived from other fields in this struct. In
    /// advanced building configurations, the default logic to derive these lines may not
    /// be sufficient. This field can be set to `Some(true)` to suppress the emission
    /// of these lines.
    ///
    /// If suppression is enabled, `extra_build_script_lines` should contain equivalent
    /// functionality or else a build failure is likely.
    pub suppress_build_script_link_lines: bool,

    /// Additional lines to `println!()` from Cargo build scripts.
    ///
    /// This field can be populated to enable the `pyo3` crate to emit additional lines from its
    /// its Cargo build script.
    ///
    /// This crate doesn't populate this field itself. Rather, it is intended to be used with
    /// externally provided config files to give them significant control over how the crate
    /// is build/configured.
    ///
    /// Serialized to multiple `extra_build_script_line` values.
    pub extra_build_script_lines: Vec<String>,
    /// macOS Python3.framework requires special rpath handling
    pub python_framework_prefix: Option<String>,
}

impl InterpreterConfig {
    #[doc(hidden)]
    pub fn build_script_outputs(&self) -> Vec<String> {
        // This should have been checked during pyo3-build-config build time.
        assert!(self.version >= MINIMUM_SUPPORTED_VERSION);

        let mut out = vec![];

        for i in MINIMUM_SUPPORTED_VERSION.minor..=self.version.minor {
            out.push(format!("cargo:rustc-cfg=Py_3_{i}"));
        }

        match self.implementation {
            PythonImplementation::CPython => {}
            PythonImplementation::PyPy => out.push("cargo:rustc-cfg=PyPy".to_owned()),
            PythonImplementation::GraalPy => out.push("cargo:rustc-cfg=GraalPy".to_owned()),
        }

        // If Py_GIL_DISABLED is set, do not build with limited API support
        if self.abi3 && !self.is_free_threaded() {
            out.push("cargo:rustc-cfg=Py_LIMITED_API".to_owned());
        }

        for flag in &self.build_flags.0 {
            match flag {
                BuildFlag::Py_GIL_DISABLED => {
                    out.push("cargo:rustc-cfg=Py_GIL_DISABLED".to_owned())
                }
                flag => out.push(format!("cargo:rustc-cfg=py_sys_config=\"{flag}\"")),
            }
        }

        out
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
from sysconfig import get_config_var, get_platform

PYPY = platform.python_implementation() == "PyPy"
GRAALPY = platform.python_implementation() == "GraalVM"

if GRAALPY:
    graalpy_ver = map(int, __graalpython__.get_graalvm_version().split('.'));
    print("graalpy_major", next(graalpy_ver))
    print("graalpy_minor", next(graalpy_ver))

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
WINDOWS = platform.system() == "Windows"

# macOS framework packages use shared linking
FRAMEWORK = bool(get_config_var("PYTHONFRAMEWORK"))
FRAMEWORK_PREFIX = get_config_var("PYTHONFRAMEWORKPREFIX")

# unix-style shared library enabled
SHARED = bool(get_config_var("Py_ENABLE_SHARED"))

print("implementation", platform.python_implementation())
print("version_major", sys.version_info[0])
print("version_minor", sys.version_info[1])
print("shared", PYPY or GRAALPY or ANACONDA or WINDOWS or FRAMEWORK or SHARED)
print("python_framework_prefix", FRAMEWORK_PREFIX)
print_if_set("ld_version", get_config_var("LDVERSION"))
print_if_set("libdir", get_config_var("LIBDIR"))
print_if_set("base_prefix", base_prefix)
print("executable", sys.executable)
print("calcsize_pointer", struct.calcsize("P"))
print("mingw", get_platform().startswith("mingw"))
print("ext_suffix", get_config_var("EXT_SUFFIX"))
print("gil_disabled", get_config_var("Py_GIL_DISABLED"))
"#;
        let output = run_python_script(interpreter.as_ref(), SCRIPT)?;
        let map: HashMap<String, String> = parse_script_output(&output);

        ensure!(
            !map.is_empty(),
            "broken Python interpreter: {}",
            interpreter.as_ref().display()
        );

        if let Some(value) = map.get("graalpy_major") {
            let graalpy_version = PythonVersion {
                major: value
                    .parse()
                    .context("failed to parse GraalPy major version")?,
                minor: map["graalpy_minor"]
                    .parse()
                    .context("failed to parse GraalPy minor version")?,
            };
            ensure!(
                graalpy_version >= MINIMUM_SUPPORTED_VERSION_GRAALPY,
                "At least GraalPy version {} needed, got {}",
                MINIMUM_SUPPORTED_VERSION_GRAALPY,
                graalpy_version
            );
        };

        let shared = map["shared"].as_str() == "True";
        let python_framework_prefix = map.get("python_framework_prefix").cloned();

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

        let gil_disabled = match map["gil_disabled"].as_str() {
            "1" => true,
            "0" => false,
            "None" => false,
            _ => panic!("Unknown Py_GIL_DISABLED value"),
        };

        let lib_name = if cfg!(windows) {
            default_lib_name_windows(
                version,
                implementation,
                abi3,
                map["mingw"].as_str() == "True",
                // This is the best heuristic currently available to detect debug build
                // on Windows from sysconfig - e.g. ext_suffix may be
                // `_d.cp312-win_amd64.pyd` for 3.12 debug build
                map["ext_suffix"].starts_with("_d."),
                gil_disabled,
            )?
        } else {
            default_lib_name_unix(
                version,
                implementation,
                map.get("ld_version").map(String::as_str),
                gil_disabled,
            )?
        };

        let lib_dir = if cfg!(windows) {
            map.get("base_prefix")
                .map(|base_prefix| format!("{base_prefix}\\libs"))
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
            build_flags: BuildFlags::from_interpreter(interpreter)?,
            suppress_build_script_link_lines: false,
            extra_build_script_lines: vec![],
            python_framework_prefix,
        })
    }

    /// Generate from parsed sysconfigdata file
    ///
    /// Use [`parse_sysconfigdata`] to generate a hash map of configuration values which may be
    /// used to build an [`InterpreterConfig`].
    pub fn from_sysconfigdata(sysconfigdata: &Sysconfigdata) -> Result<Self> {
        macro_rules! get_key {
            ($sysconfigdata:expr, $key:literal) => {
                $sysconfigdata
                    .get_value($key)
                    .ok_or(concat!($key, " not found in sysconfigdata file"))
            };
        }

        macro_rules! parse_key {
            ($sysconfigdata:expr, $key:literal) => {
                get_key!($sysconfigdata, $key)?
                    .parse()
                    .context(concat!("could not parse value of ", $key))
            };
        }

        let soabi = get_key!(sysconfigdata, "SOABI")?;
        let implementation = PythonImplementation::from_soabi(soabi)?;
        let version = parse_key!(sysconfigdata, "VERSION")?;
        let shared = match sysconfigdata.get_value("Py_ENABLE_SHARED") {
            Some("1") | Some("true") | Some("True") => true,
            Some("0") | Some("false") | Some("False") => false,
            _ => bail!("expected a bool (1/true/True or 0/false/False) for Py_ENABLE_SHARED"),
        };
        // macOS framework packages use shared linking (PYTHONFRAMEWORK is the framework name, hence the empty check)
        let framework = match sysconfigdata.get_value("PYTHONFRAMEWORK") {
            Some(s) => !s.is_empty(),
            _ => false,
        };
        let python_framework_prefix = sysconfigdata
            .get_value("PYTHONFRAMEWORKPREFIX")
            .map(str::to_string);
        let lib_dir = get_key!(sysconfigdata, "LIBDIR").ok().map(str::to_string);
        let gil_disabled = match sysconfigdata.get_value("Py_GIL_DISABLED") {
            Some(value) => value == "1",
            None => false,
        };
        let lib_name = Some(default_lib_name_unix(
            version,
            implementation,
            sysconfigdata.get_value("LDVERSION"),
            gil_disabled,
        )?);
        let pointer_width = parse_key!(sysconfigdata, "SIZEOF_VOID_P")
            .map(|bytes_width: u32| bytes_width * 8)
            .ok();
        let build_flags = BuildFlags::from_sysconfigdata(sysconfigdata);

        Ok(InterpreterConfig {
            implementation,
            version,
            shared: shared || framework,
            abi3: is_abi3(),
            lib_dir,
            lib_name,
            executable: None,
            pointer_width,
            build_flags,
            suppress_build_script_link_lines: false,
            extra_build_script_lines: vec![],
            python_framework_prefix,
        })
    }

    /// Import an externally-provided config file.
    ///
    /// The `abi3` features, if set, may apply an `abi3` constraint to the Python version.
    #[allow(dead_code)] // only used in build.rs
    pub(super) fn from_pyo3_config_file_env() -> Option<Result<Self>> {
        env_var("PYO3_CONFIG_FILE").map(|path| {
            let path = Path::new(&path);
            println!("cargo:rerun-if-changed={}", path.display());
            // Absolute path is necessary because this build script is run with a cwd different to the
            // original `cargo build` instruction.
            ensure!(
                path.is_absolute(),
                "PYO3_CONFIG_FILE must be an absolute path"
            );

            let mut config = InterpreterConfig::from_path(path)
                .context("failed to parse contents of PYO3_CONFIG_FILE")?;
            // If the abi3 feature is enabled, the minimum Python version is constrained by the abi3
            // feature.
            //
            // TODO: abi3 is a property of the build mode, not the interpreter. Should this be
            // removed from `InterpreterConfig`?
            config.abi3 |= is_abi3();
            config.fixup_for_abi3_version(get_abi3_version())?;

            Ok(config)
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
    pub fn from_cargo_dep_env() -> Option<Result<Self>> {
        cargo_env_var("DEP_PYTHON_PYO3_CONFIG")
            .map(|buf| InterpreterConfig::from_reader(&*unescape(&buf)))
    }

    #[doc(hidden)]
    pub fn from_reader(reader: impl Read) -> Result<Self> {
        let reader = BufReader::new(reader);
        let lines = reader.lines();

        macro_rules! parse_value {
            ($variable:ident, $value:ident) => {
                $variable = Some($value.trim().parse().context(format!(
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
        let mut build_flags: Option<BuildFlags> = None;
        let mut suppress_build_script_link_lines = None;
        let mut extra_build_script_lines = vec![];
        let mut python_framework_prefix = None;

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
                "suppress_build_script_link_lines" => {
                    parse_value!(suppress_build_script_link_lines, value)
                }
                "extra_build_script_line" => {
                    extra_build_script_lines.push(value.to_string());
                }
                "python_framework_prefix" => parse_value!(python_framework_prefix, value),
                unknown => warn!("unknown config key `{}`", unknown),
            }
        }

        let version = version.ok_or("missing value for version")?;
        let implementation = implementation.unwrap_or(PythonImplementation::CPython);
        let abi3 = abi3.unwrap_or(false);
        let build_flags = build_flags.unwrap_or_default();
        let gil_disabled = build_flags.0.contains(&BuildFlag::Py_GIL_DISABLED);
        // Fixup lib_name if it's not set
        let lib_name = lib_name.or_else(|| {
            if let Ok(Ok(target)) = env::var("TARGET").map(|target| target.parse::<Triple>()) {
                default_lib_name_for_target(version, implementation, abi3, gil_disabled, &target)
            } else {
                None
            }
        });

        Ok(InterpreterConfig {
            implementation,
            version,
            shared: shared.unwrap_or(true),
            abi3,
            lib_name,
            lib_dir,
            executable,
            pointer_width,
            build_flags,
            suppress_build_script_link_lines: suppress_build_script_link_lines.unwrap_or(false),
            extra_build_script_lines,
            python_framework_prefix,
        })
    }

    #[cfg(feature = "python3-dll-a")]
    #[allow(clippy::unnecessary_wraps)]
    pub fn generate_import_libs(&mut self) -> Result<()> {
        // Auto generate python3.dll import libraries for Windows targets.
        if self.lib_dir.is_none() {
            let target = target_triple_from_env();
            let py_version = if self.implementation == PythonImplementation::CPython
                && self.abi3
                && !self.is_free_threaded()
            {
                None
            } else {
                Some(self.version)
            };
            let abiflags = if self.is_free_threaded() {
                Some("t")
            } else {
                None
            };
            self.lib_dir = import_lib::generate_import_lib(
                &target,
                self.implementation,
                py_version,
                abiflags,
            )?;
        }
        Ok(())
    }

    #[cfg(not(feature = "python3-dll-a"))]
    #[allow(clippy::unnecessary_wraps)]
    pub fn generate_import_libs(&mut self) -> Result<()> {
        Ok(())
    }

    #[doc(hidden)]
    /// Serialize the `InterpreterConfig` and print it to the environment for Cargo to pass along
    /// to dependent packages during build time.
    ///
    /// NB: writing to the cargo environment requires the
    /// [`links`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#the-links-manifest-key)
    /// manifest key to be set. In this case that means this is called by the `pyo3-ffi` crate and
    /// available for dependent package build scripts in `DEP_PYTHON_PYO3_CONFIG`. See
    /// documentation for the
    /// [`DEP_<name>_<key>`](https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts)
    /// environment variable.
    pub fn to_cargo_dep_env(&self) -> Result<()> {
        let mut buf = Vec::new();
        self.to_writer(&mut buf)?;
        // escape newlines in env var
        println!("cargo:PYO3_CONFIG={}", escape(&buf));
        Ok(())
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
        write_option_line!(python_framework_prefix)?;
        write_line!(suppress_build_script_link_lines)?;
        for line in &self.extra_build_script_lines {
            writeln!(writer, "extra_build_script_line={line}")
                .context("failed to write extra_build_script_line")?;
        }
        Ok(())
    }

    /// Run a python script using the [`InterpreterConfig::executable`].
    ///
    /// # Panics
    ///
    /// This function will panic if the [`executable`](InterpreterConfig::executable) is `None`.
    pub fn run_python_script(&self, script: &str) -> Result<String> {
        run_python_script_with_envs(
            Path::new(self.executable.as_ref().expect("no interpreter executable")),
            script,
            std::iter::empty::<(&str, &str)>(),
        )
    }

    /// Run a python script using the [`InterpreterConfig::executable`] with additional
    /// environment variables (e.g. PYTHONPATH) set.
    ///
    /// # Panics
    ///
    /// This function will panic if the [`executable`](InterpreterConfig::executable) is `None`.
    pub fn run_python_script_with_envs<I, K, V>(&self, script: &str, envs: I) -> Result<String>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        run_python_script_with_envs(
            Path::new(self.executable.as_ref().expect("no interpreter executable")),
            script,
            envs,
        )
    }

    pub fn is_free_threaded(&self) -> bool {
        self.build_flags.0.contains(&BuildFlag::Py_GIL_DISABLED)
    }

    /// Updates configured ABI to build for to the requested abi3 version
    /// This is a no-op for platforms where abi3 is not supported
    fn fixup_for_abi3_version(&mut self, abi3_version: Option<PythonVersion>) -> Result<()> {
        // PyPy, GraalPy, and the free-threaded build don't support abi3; don't adjust the version
        if self.implementation.is_pypy()
            || self.implementation.is_graalpy()
            || self.is_free_threaded()
        {
            return Ok(());
        }

        if let Some(version) = abi3_version {
            ensure!(
                version <= self.version,
                "cannot set a minimum Python version {} higher than the interpreter version {} \
                (the minimum Python version is implied by the abi3-py3{} feature)",
                version,
                self.version,
                version.minor,
            );

            self.version = version;
        } else if is_abi3() && self.version.minor > ABI3_MAX_MINOR {
            warn!("Automatically falling back to abi3-py3{ABI3_MAX_MINOR} because current Python is higher than the maximum supported");
            self.version.minor = ABI3_MAX_MINOR;
        }

        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PythonVersion {
    pub major: u8,
    pub minor: u8,
}

impl PythonVersion {
    pub const PY313: Self = PythonVersion {
        major: 3,
        minor: 13,
    };
    const PY310: Self = PythonVersion {
        major: 3,
        minor: 10,
    };
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PythonImplementation {
    CPython,
    PyPy,
    GraalPy,
}

impl PythonImplementation {
    #[doc(hidden)]
    pub fn is_pypy(self) -> bool {
        self == PythonImplementation::PyPy
    }

    #[doc(hidden)]
    pub fn is_graalpy(self) -> bool {
        self == PythonImplementation::GraalPy
    }

    #[doc(hidden)]
    pub fn from_soabi(soabi: &str) -> Result<Self> {
        if soabi.starts_with("pypy") {
            Ok(PythonImplementation::PyPy)
        } else if soabi.starts_with("cpython") {
            Ok(PythonImplementation::CPython)
        } else if soabi.starts_with("graalpy") {
            Ok(PythonImplementation::GraalPy)
        } else {
            bail!("unsupported Python interpreter");
        }
    }
}

impl Display for PythonImplementation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PythonImplementation::CPython => write!(f, "CPython"),
            PythonImplementation::PyPy => write!(f, "PyPy"),
            PythonImplementation::GraalPy => write!(f, "GraalVM"),
        }
    }
}

impl FromStr for PythonImplementation {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "CPython" => Ok(PythonImplementation::CPython),
            "PyPy" => Ok(PythonImplementation::PyPy),
            "GraalVM" => Ok(PythonImplementation::GraalPy),
            _ => bail!("unknown interpreter: {}", s),
        }
    }
}

/// Checks if we should look for a Python interpreter installation
/// to get the target interpreter configuration.
///
/// Returns `false` if `PYO3_NO_PYTHON` environment variable is set.
fn have_python_interpreter() -> bool {
    env_var("PYO3_NO_PYTHON").is_none()
}

/// Checks if `abi3` or any of the `abi3-py3*` features is enabled for the PyO3 crate.
///
/// Must be called from a PyO3 crate build script.
fn is_abi3() -> bool {
    cargo_env_var("CARGO_FEATURE_ABI3").is_some()
        || env_var("PYO3_USE_ABI3_FORWARD_COMPATIBILITY").is_some_and(|os_str| os_str == "1")
}

/// Gets the minimum supported Python version from PyO3 `abi3-py*` features.
///
/// Must be called from a PyO3 crate build script.
pub fn get_abi3_version() -> Option<PythonVersion> {
    let minor_version = (MINIMUM_SUPPORTED_VERSION.minor..=ABI3_MAX_MINOR)
        .find(|i| cargo_env_var(&format!("CARGO_FEATURE_ABI3_PY3{i}")).is_some());
    minor_version.map(|minor| PythonVersion { major: 3, minor })
}

/// Checks if the `extension-module` feature is enabled for the PyO3 crate.
///
/// This can be triggered either by:
/// - The `extension-module` Cargo feature
/// - Setting the `PYO3_BUILD_EXTENSION_MODULE` environment variable
///
/// Must be called from a PyO3 crate build script.
pub fn is_extension_module() -> bool {
    cargo_env_var("CARGO_FEATURE_EXTENSION_MODULE").is_some()
        || env_var("PYO3_BUILD_EXTENSION_MODULE").is_some()
}

/// Checks if we need to link to `libpython` for the current build target.
///
/// Must be called from a PyO3 crate build script.
pub fn is_linking_libpython() -> bool {
    is_linking_libpython_for_target(&target_triple_from_env())
}

/// Checks if we need to link to `libpython` for the target.
///
/// Must be called from a PyO3 crate build script.
fn is_linking_libpython_for_target(target: &Triple) -> bool {
    target.operating_system == OperatingSystem::Windows
        // See https://github.com/PyO3/pyo3/issues/4068#issuecomment-2051159852
        || target.operating_system == OperatingSystem::Aix
        || target.environment == Environment::Android
        || target.environment == Environment::Androideabi
        || !is_extension_module()
}

/// Checks if we need to discover the Python library directory
/// to link the extension module binary.
///
/// Must be called from a PyO3 crate build script.
fn require_libdir_for_target(target: &Triple) -> bool {
    let is_generating_libpython = cfg!(feature = "python3-dll-a")
        && target.operating_system == OperatingSystem::Windows
        && is_abi3();

    is_linking_libpython_for_target(target) && !is_generating_libpython
}

/// Configuration needed by PyO3 to cross-compile for a target platform.
///
/// Usually this is collected from the environment (i.e. `PYO3_CROSS_*` and `CARGO_CFG_TARGET_*`)
/// when a cross-compilation configuration is detected.
#[derive(Debug, PartialEq, Eq)]
pub struct CrossCompileConfig {
    /// The directory containing the Python library to link against.
    pub lib_dir: Option<PathBuf>,

    /// The version of the Python library to link against.
    version: Option<PythonVersion>,

    /// The target Python implementation hint (CPython, PyPy, GraalPy, ...)
    implementation: Option<PythonImplementation>,

    /// The compile target triple (e.g. aarch64-unknown-linux-gnu)
    target: Triple,

    /// Python ABI flags, used to detect free-threaded Python builds.
    abiflags: Option<String>,
}

impl CrossCompileConfig {
    /// Creates a new cross compile config struct from PyO3 environment variables
    /// and the build environment when cross compilation mode is detected.
    ///
    /// Returns `None` when not cross compiling.
    fn try_from_env_vars_host_target(
        env_vars: CrossCompileEnvVars,
        host: &Triple,
        target: &Triple,
    ) -> Result<Option<Self>> {
        if env_vars.any() || Self::is_cross_compiling_from_to(host, target) {
            let lib_dir = env_vars.lib_dir_path()?;
            let (version, abiflags) = env_vars.parse_version()?;
            let implementation = env_vars.parse_implementation()?;
            let target = target.clone();

            Ok(Some(CrossCompileConfig {
                lib_dir,
                version,
                implementation,
                target,
                abiflags,
            }))
        } else {
            Ok(None)
        }
    }

    /// Checks if compiling on `host` for `target` required "real" cross compilation.
    ///
    /// Returns `false` if the target Python interpreter can run on the host.
    fn is_cross_compiling_from_to(host: &Triple, target: &Triple) -> bool {
        // Not cross-compiling if arch-vendor-os is all the same
        // e.g. x86_64-unknown-linux-musl on x86_64-unknown-linux-gnu host
        //      x86_64-pc-windows-gnu on x86_64-pc-windows-msvc host
        let mut compatible = host.architecture == target.architecture
            && (host.vendor == target.vendor
                // Don't treat `-pc-` to `-win7-` as cross-compiling
                || (host.vendor == Vendor::Pc && target.vendor.as_str() == "win7"))
            && host.operating_system == target.operating_system;

        // Not cross-compiling to compile for 32-bit Python from windows 64-bit
        compatible |= target.operating_system == OperatingSystem::Windows
            && host.operating_system == OperatingSystem::Windows
            && matches!(target.architecture, Architecture::X86_32(_))
            && host.architecture == Architecture::X86_64;

        // Not cross-compiling to compile for x86-64 Python from macOS arm64 and vice versa
        compatible |= matches!(target.operating_system, OperatingSystem::Darwin(_))
            && matches!(host.operating_system, OperatingSystem::Darwin(_));

        !compatible
    }

    /// Converts `lib_dir` member field to an UTF-8 string.
    ///
    /// The conversion can not fail because `PYO3_CROSS_LIB_DIR` variable
    /// is ensured contain a valid UTF-8 string.
    fn lib_dir_string(&self) -> Option<String> {
        self.lib_dir
            .as_ref()
            .map(|s| s.to_str().unwrap().to_owned())
    }
}

/// PyO3-specific cross compile environment variable values
struct CrossCompileEnvVars {
    /// `PYO3_CROSS`
    pyo3_cross: Option<OsString>,
    /// `PYO3_CROSS_LIB_DIR`
    pyo3_cross_lib_dir: Option<OsString>,
    /// `PYO3_CROSS_PYTHON_VERSION`
    pyo3_cross_python_version: Option<OsString>,
    /// `PYO3_CROSS_PYTHON_IMPLEMENTATION`
    pyo3_cross_python_implementation: Option<OsString>,
}

impl CrossCompileEnvVars {
    /// Grabs the PyO3 cross-compile variables from the environment.
    ///
    /// Registers the build script to rerun if any of the variables changes.
    fn from_env() -> Self {
        CrossCompileEnvVars {
            pyo3_cross: env_var("PYO3_CROSS"),
            pyo3_cross_lib_dir: env_var("PYO3_CROSS_LIB_DIR"),
            pyo3_cross_python_version: env_var("PYO3_CROSS_PYTHON_VERSION"),
            pyo3_cross_python_implementation: env_var("PYO3_CROSS_PYTHON_IMPLEMENTATION"),
        }
    }

    /// Checks if any of the variables is set.
    fn any(&self) -> bool {
        self.pyo3_cross.is_some()
            || self.pyo3_cross_lib_dir.is_some()
            || self.pyo3_cross_python_version.is_some()
            || self.pyo3_cross_python_implementation.is_some()
    }

    /// Parses `PYO3_CROSS_PYTHON_VERSION` environment variable value
    /// into `PythonVersion` and ABI flags.
    fn parse_version(&self) -> Result<(Option<PythonVersion>, Option<String>)> {
        match self.pyo3_cross_python_version.as_ref() {
            Some(os_string) => {
                let utf8_str = os_string
                    .to_str()
                    .ok_or("PYO3_CROSS_PYTHON_VERSION is not valid a UTF-8 string")?;
                let (utf8_str, abiflags) = if let Some(version) = utf8_str.strip_suffix('t') {
                    (version, Some("t".to_string()))
                } else {
                    (utf8_str, None)
                };
                let version = utf8_str
                    .parse()
                    .context("failed to parse PYO3_CROSS_PYTHON_VERSION")?;
                Ok((Some(version), abiflags))
            }
            None => Ok((None, None)),
        }
    }

    /// Parses `PYO3_CROSS_PYTHON_IMPLEMENTATION` environment variable value
    /// into `PythonImplementation`.
    fn parse_implementation(&self) -> Result<Option<PythonImplementation>> {
        let implementation = self
            .pyo3_cross_python_implementation
            .as_ref()
            .map(|os_string| {
                let utf8_str = os_string
                    .to_str()
                    .ok_or("PYO3_CROSS_PYTHON_IMPLEMENTATION is not valid a UTF-8 string")?;
                utf8_str
                    .parse()
                    .context("failed to parse PYO3_CROSS_PYTHON_IMPLEMENTATION")
            })
            .transpose()?;

        Ok(implementation)
    }

    /// Converts the stored `PYO3_CROSS_LIB_DIR` variable value (if any)
    /// into a `PathBuf` instance.
    ///
    /// Ensures that the path is a valid UTF-8 string.
    fn lib_dir_path(&self) -> Result<Option<PathBuf>> {
        let lib_dir = self.pyo3_cross_lib_dir.as_ref().map(PathBuf::from);

        if let Some(dir) = lib_dir.as_ref() {
            ensure!(
                dir.to_str().is_some(),
                "PYO3_CROSS_LIB_DIR variable value is not a valid UTF-8 string"
            );
        }

        Ok(lib_dir)
    }
}

/// Detect whether we are cross compiling and return an assembled CrossCompileConfig if so.
///
/// This function relies on PyO3 cross-compiling environment variables:
///
/// * `PYO3_CROSS`: If present, forces PyO3 to configure as a cross-compilation.
/// * `PYO3_CROSS_LIB_DIR`: If present, must be set to the directory containing
///   the target's libpython DSO and the associated `_sysconfigdata*.py` file for
///   Unix-like targets, or the Python DLL import libraries for the Windows target.
/// * `PYO3_CROSS_PYTHON_VERSION`: Major and minor version (e.g. 3.9) of the target Python
///   installation. This variable is only needed if PyO3 cannnot determine the version to target
///   from `abi3-py3*` features, or if there are multiple versions of Python present in
///   `PYO3_CROSS_LIB_DIR`.
///
/// See the [PyO3 User Guide](https://pyo3.rs/) for more info on cross-compiling.
pub fn cross_compiling_from_to(
    host: &Triple,
    target: &Triple,
) -> Result<Option<CrossCompileConfig>> {
    let env_vars = CrossCompileEnvVars::from_env();
    CrossCompileConfig::try_from_env_vars_host_target(env_vars, host, target)
}

/// Detect whether we are cross compiling from Cargo and `PYO3_CROSS_*` environment
/// variables and return an assembled `CrossCompileConfig` if so.
///
/// This must be called from PyO3's build script, because it relies on environment
/// variables such as `CARGO_CFG_TARGET_OS` which aren't available at any other time.
pub fn cross_compiling_from_cargo_env() -> Result<Option<CrossCompileConfig>> {
    let env_vars = CrossCompileEnvVars::from_env();
    let host = Triple::host();
    let target = target_triple_from_env();

    CrossCompileConfig::try_from_env_vars_host_target(env_vars, &host, &target)
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum BuildFlag {
    Py_DEBUG,
    Py_REF_DEBUG,
    Py_TRACE_REFS,
    Py_GIL_DISABLED,
    COUNT_ALLOCS,
    Other(String),
}

impl Display for BuildFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildFlag::Other(flag) => write!(f, "{flag}"),
            _ => write!(f, "{self:?}"),
        }
    }
}

impl FromStr for BuildFlag {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Py_DEBUG" => Ok(BuildFlag::Py_DEBUG),
            "Py_REF_DEBUG" => Ok(BuildFlag::Py_REF_DEBUG),
            "Py_TRACE_REFS" => Ok(BuildFlag::Py_TRACE_REFS),
            "Py_GIL_DISABLED" => Ok(BuildFlag::Py_GIL_DISABLED),
            "COUNT_ALLOCS" => Ok(BuildFlag::COUNT_ALLOCS),
            other => Ok(BuildFlag::Other(other.to_owned())),
        }
    }
}

/// A list of python interpreter compile-time preprocessor defines.
///
/// PyO3 will pick these up and pass to rustc via `--cfg=py_sys_config={varname}`;
/// this allows using them conditional cfg attributes in the .rs files, so
///
/// ```rust,no_run
/// #[cfg(py_sys_config="{varname}")]
/// # struct Foo;
/// ```
///
/// is the equivalent of `#ifdef {varname}` in C.
///
/// see Misc/SpecialBuilds.txt in the python source for what these mean.
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[derive(Clone, Default)]
pub struct BuildFlags(pub HashSet<BuildFlag>);

impl BuildFlags {
    const ALL: [BuildFlag; 5] = [
        BuildFlag::Py_DEBUG,
        BuildFlag::Py_REF_DEBUG,
        BuildFlag::Py_TRACE_REFS,
        BuildFlag::Py_GIL_DISABLED,
        BuildFlag::COUNT_ALLOCS,
    ];

    pub fn new() -> Self {
        BuildFlags(HashSet::new())
    }

    fn from_sysconfigdata(config_map: &Sysconfigdata) -> Self {
        Self(
            BuildFlags::ALL
                .iter()
                .filter(|flag| config_map.get_value(flag.to_string()) == Some("1"))
                .cloned()
                .collect(),
        )
        .fixup()
    }

    /// Examine python's compile flags to pass to cfg by launching
    /// the interpreter and printing variables of interest from
    /// sysconfig.get_config_vars.
    fn from_interpreter(interpreter: impl AsRef<Path>) -> Result<Self> {
        // sysconfig is missing all the flags on windows for Python 3.12 and
        // older, so we can't actually query the interpreter directly for its
        // build flags on those versions.
        if cfg!(windows) {
            let script = String::from("import sys;print(sys.version_info < (3, 13))");
            let stdout = run_python_script(interpreter.as_ref(), &script)?;
            if stdout.trim_end() == "True" {
                return Ok(Self::new());
            }
        }

        let mut script = String::from("import sysconfig\n");
        script.push_str("config = sysconfig.get_config_vars()\n");

        for k in &BuildFlags::ALL {
            use std::fmt::Write;
            writeln!(&mut script, "print(config.get('{k}', '0'))").unwrap();
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

        Ok(Self(flags).fixup())
    }

    fn fixup(mut self) -> Self {
        if self.0.contains(&BuildFlag::Py_DEBUG) {
            self.0.insert(BuildFlag::Py_REF_DEBUG);
        }

        self
    }
}

impl Display for BuildFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for flag in &self.0 {
            if first {
                first = false;
            } else {
                write!(f, ",")?;
            }
            write!(f, "{flag}")?;
        }
        Ok(())
    }
}

impl FromStr for BuildFlags {
    type Err = std::convert::Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut flags = HashSet::new();
        for flag in value.split_terminator(',') {
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

/// Parsed data from Python sysconfigdata file
///
/// A hash map of all values from a sysconfigdata file.
pub struct Sysconfigdata(HashMap<String, String>);

impl Sysconfigdata {
    pub fn get_value<S: AsRef<str>>(&self, k: S) -> Option<&str> {
        self.0.get(k.as_ref()).map(String::as_str)
    }

    #[allow(dead_code)]
    fn new() -> Self {
        Sysconfigdata(HashMap::new())
    }

    #[allow(dead_code)]
    fn insert<S: Into<String>>(&mut self, k: S, v: S) {
        self.0.insert(k.into(), v.into());
    }
}

/// Parse sysconfigdata file
///
/// The sysconfigdata is simply a dictionary containing all the build time variables used for the
/// python executable and library. This function necessitates a python interpreter on the host
/// machine to work. Here it is read into a `Sysconfigdata` (hash map), which can be turned into an
/// [`InterpreterConfig`] using
/// [`from_sysconfigdata`](InterpreterConfig::from_sysconfigdata).
pub fn parse_sysconfigdata(sysconfigdata_path: impl AsRef<Path>) -> Result<Sysconfigdata> {
    let sysconfigdata_path = sysconfigdata_path.as_ref();
    let mut script = fs::read_to_string(sysconfigdata_path).with_context(|| {
        format!(
            "failed to read config from {}",
            sysconfigdata_path.display()
        )
    })?;
    script += r#"
for key, val in build_time_vars.items():
    # (ana)conda(-forge) built Pythons are statically linked but ship the shared library with them.
    # We detect them based on the magic prefix directory they have encoded in their builds.
    if key == "Py_ENABLE_SHARED" and "_h_env_placehold" in build_time_vars.get("prefix"):
        val = 1
    print(key, val)
"#;

    let output = run_python_script(&find_interpreter()?, &script)?;

    Ok(Sysconfigdata(parse_script_output(&output)))
}

fn starts_with(entry: &DirEntry, pat: &str) -> bool {
    let name = entry.file_name();
    name.to_string_lossy().starts_with(pat)
}
fn ends_with(entry: &DirEntry, pat: &str) -> bool {
    let name = entry.file_name();
    name.to_string_lossy().ends_with(pat)
}

/// Finds the sysconfigdata file when the target Python library directory is set.
///
/// Returns `None` if the library directory is not available, and a runtime error
/// when no or multiple sysconfigdata files are found.
fn find_sysconfigdata(cross: &CrossCompileConfig) -> Result<Option<PathBuf>> {
    let mut sysconfig_paths = find_all_sysconfigdata(cross)?;
    if sysconfig_paths.is_empty() {
        if let Some(lib_dir) = cross.lib_dir.as_ref() {
            bail!("Could not find _sysconfigdata*.py in {}", lib_dir.display());
        } else {
            // Continue with the default configuration when PYO3_CROSS_LIB_DIR is not set.
            return Ok(None);
        }
    } else if sysconfig_paths.len() > 1 {
        let mut error_msg = String::from(
            "Detected multiple possible Python versions. Please set either the \
            PYO3_CROSS_PYTHON_VERSION variable to the wanted version or the \
            _PYTHON_SYSCONFIGDATA_NAME variable to the wanted sysconfigdata file name.\n\n\
            sysconfigdata files found:",
        );
        for path in sysconfig_paths {
            use std::fmt::Write;
            write!(&mut error_msg, "\n\t{}", path.display()).unwrap();
        }
        bail!("{}\n", error_msg);
    }

    Ok(Some(sysconfig_paths.remove(0)))
}

/// Finds `_sysconfigdata*.py` files for detected Python interpreters.
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
/// # distribution from package manager, (lib_dir may or may not include lib/)
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
///
/// # PyPy includes a similar file since v73
/// ${INSTALL_PREFIX}/lib/pypy3.Y/_sysconfigdata.py
/// ${INSTALL_PREFIX}/lib_pypy/_sysconfigdata.py
/// ```
///
/// [1]: https://github.com/python/cpython/blob/3.5/Lib/sysconfig.py#L389
///
/// Returns an empty vector when the target Python library directory
/// is not set via `PYO3_CROSS_LIB_DIR`.
pub fn find_all_sysconfigdata(cross: &CrossCompileConfig) -> Result<Vec<PathBuf>> {
    let sysconfig_paths = if let Some(lib_dir) = cross.lib_dir.as_ref() {
        search_lib_dir(lib_dir, cross).with_context(|| {
            format!(
                "failed to search the lib dir at 'PYO3_CROSS_LIB_DIR={}'",
                lib_dir.display()
            )
        })?
    } else {
        return Ok(Vec::new());
    };

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

    sysconfig_paths.sort();
    sysconfig_paths.dedup();

    Ok(sysconfig_paths)
}

fn is_pypy_lib_dir(path: &str, v: &Option<PythonVersion>) -> bool {
    let pypy_version_pat = if let Some(v) = v {
        format!("pypy{v}")
    } else {
        "pypy3.".into()
    };
    path == "lib_pypy" || path.starts_with(&pypy_version_pat)
}

fn is_graalpy_lib_dir(path: &str, v: &Option<PythonVersion>) -> bool {
    let graalpy_version_pat = if let Some(v) = v {
        format!("graalpy{v}")
    } else {
        "graalpy2".into()
    };
    path == "lib_graalpython" || path.starts_with(&graalpy_version_pat)
}

fn is_cpython_lib_dir(path: &str, v: &Option<PythonVersion>) -> bool {
    let cpython_version_pat = if let Some(v) = v {
        format!("python{v}")
    } else {
        "python3.".into()
    };
    path.starts_with(&cpython_version_pat)
}

/// recursive search for _sysconfigdata, returns all possibilities of sysconfigdata paths
fn search_lib_dir(path: impl AsRef<Path>, cross: &CrossCompileConfig) -> Result<Vec<PathBuf>> {
    let mut sysconfig_paths = vec![];
    for f in fs::read_dir(path.as_ref()).with_context(|| {
        format!(
            "failed to list the entries in '{}'",
            path.as_ref().display()
        )
    })? {
        sysconfig_paths.extend(match &f {
            // Python 3.7+ sysconfigdata with platform specifics
            Ok(f) if starts_with(f, "_sysconfigdata_") && ends_with(f, "py") => vec![f.path()],
            Ok(f) if f.metadata().is_ok_and(|metadata| metadata.is_dir()) => {
                let file_name = f.file_name();
                let file_name = file_name.to_string_lossy();
                if file_name == "build" || file_name == "lib" {
                    search_lib_dir(f.path(), cross)?
                } else if file_name.starts_with("lib.") {
                    // check if right target os
                    if !file_name.contains(&cross.target.operating_system.to_string()) {
                        continue;
                    }
                    // Check if right arch
                    if !file_name.contains(&cross.target.architecture.to_string()) {
                        continue;
                    }
                    search_lib_dir(f.path(), cross)?
                } else if is_cpython_lib_dir(&file_name, &cross.version)
                    || is_pypy_lib_dir(&file_name, &cross.version)
                    || is_graalpy_lib_dir(&file_name, &cross.version)
                {
                    search_lib_dir(f.path(), cross)?
                } else {
                    continue;
                }
            }
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
            .filter(|p| {
                p.to_string_lossy()
                    .contains(&cross.target.architecture.to_string())
            })
            .cloned()
            .collect::<Vec<PathBuf>>();
        if !temp.is_empty() {
            sysconfig_paths = temp;
        }
    }

    Ok(sysconfig_paths)
}

/// Find cross compilation information from sysconfigdata file
///
/// first find sysconfigdata file which follows the pattern [`_sysconfigdata_{abi}_{platform}_{multiarch}`][1]
///
/// [1]: https://github.com/python/cpython/blob/3.8/Lib/sysconfig.py#L348
///
/// Returns `None` when the target Python library directory is not set.
fn cross_compile_from_sysconfigdata(
    cross_compile_config: &CrossCompileConfig,
) -> Result<Option<InterpreterConfig>> {
    if let Some(path) = find_sysconfigdata(cross_compile_config)? {
        let data = parse_sysconfigdata(path)?;
        let mut config = InterpreterConfig::from_sysconfigdata(&data)?;
        if let Some(cross_lib_dir) = cross_compile_config.lib_dir_string() {
            config.lib_dir = Some(cross_lib_dir)
        }

        Ok(Some(config))
    } else {
        Ok(None)
    }
}

/// Generates "default" cross compilation information for the target.
///
/// This should work for most CPython extension modules when targeting
/// Windows, macOS and Linux.
///
/// Must be called from a PyO3 crate build script.
#[allow(unused_mut)]
fn default_cross_compile(cross_compile_config: &CrossCompileConfig) -> Result<InterpreterConfig> {
    let version = cross_compile_config
        .version
        .or_else(get_abi3_version)
        .ok_or_else(||
            format!(
                "PYO3_CROSS_PYTHON_VERSION or an abi3-py3* feature must be specified \
                when cross-compiling and PYO3_CROSS_LIB_DIR is not set.\n\
                = help: see the PyO3 user guide for more information: https://pyo3.rs/v{}/building-and-distribution.html#cross-compiling",
                env!("CARGO_PKG_VERSION")
            )
        )?;

    let abi3 = is_abi3();
    let implementation = cross_compile_config
        .implementation
        .unwrap_or(PythonImplementation::CPython);
    let gil_disabled = cross_compile_config.abiflags.as_deref() == Some("t");

    let lib_name = default_lib_name_for_target(
        version,
        implementation,
        abi3,
        gil_disabled,
        &cross_compile_config.target,
    );

    let mut lib_dir = cross_compile_config.lib_dir_string();

    // Auto generate python3.dll import libraries for Windows targets.
    #[cfg(feature = "python3-dll-a")]
    if lib_dir.is_none() {
        let py_version = if implementation == PythonImplementation::CPython && abi3 && !gil_disabled
        {
            None
        } else {
            Some(version)
        };
        lib_dir = self::import_lib::generate_import_lib(
            &cross_compile_config.target,
            cross_compile_config
                .implementation
                .unwrap_or(PythonImplementation::CPython),
            py_version,
            None,
        )?;
    }

    Ok(InterpreterConfig {
        implementation,
        version,
        shared: true,
        abi3,
        lib_name,
        lib_dir,
        executable: None,
        pointer_width: None,
        build_flags: BuildFlags::default(),
        suppress_build_script_link_lines: false,
        extra_build_script_lines: vec![],
        python_framework_prefix: None,
    })
}

/// Generates "default" interpreter configuration when compiling "abi3" extensions
/// without a working Python interpreter.
///
/// `version` specifies the minimum supported Stable ABI CPython version.
///
/// This should work for most CPython extension modules when compiling on
/// Windows, macOS and Linux.
///
/// Must be called from a PyO3 crate build script.
fn default_abi3_config(host: &Triple, version: PythonVersion) -> Result<InterpreterConfig> {
    // FIXME: PyPy & GraalPy do not support the Stable ABI.
    let implementation = PythonImplementation::CPython;
    let abi3 = true;

    let lib_name = if host.operating_system == OperatingSystem::Windows {
        Some(default_lib_name_windows(
            version,
            implementation,
            abi3,
            false,
            false,
            false,
        )?)
    } else {
        None
    };

    Ok(InterpreterConfig {
        implementation,
        version,
        shared: true,
        abi3,
        lib_name,
        lib_dir: None,
        executable: None,
        pointer_width: None,
        build_flags: BuildFlags::default(),
        suppress_build_script_link_lines: false,
        extra_build_script_lines: vec![],
        python_framework_prefix: None,
    })
}

/// Detects the cross compilation target interpreter configuration from all
/// available sources (PyO3 environment variables, Python sysconfigdata, etc.).
///
/// Returns the "default" target interpreter configuration for Windows and
/// when no target Python interpreter is found.
///
/// Must be called from a PyO3 crate build script.
fn load_cross_compile_config(
    cross_compile_config: CrossCompileConfig,
) -> Result<InterpreterConfig> {
    let windows = cross_compile_config.target.operating_system == OperatingSystem::Windows;

    let config = if windows || !have_python_interpreter() {
        // Load the defaults for Windows even when `PYO3_CROSS_LIB_DIR` is set
        // since it has no sysconfigdata files in it.
        // Also, do not try to look for sysconfigdata when `PYO3_NO_PYTHON` variable is set.
        default_cross_compile(&cross_compile_config)?
    } else if let Some(config) = cross_compile_from_sysconfigdata(&cross_compile_config)? {
        // Try to find and parse sysconfigdata files on other targets.
        config
    } else {
        // Fall back to the defaults when nothing else can be done.
        default_cross_compile(&cross_compile_config)?
    };

    if config.lib_name.is_some() && config.lib_dir.is_none() {
        warn!(
            "The output binary will link to libpython, \
            but PYO3_CROSS_LIB_DIR environment variable is not set. \
            Ensure that the target Python library directory is \
            in the rustc native library search path."
        );
    }

    Ok(config)
}

// These contains only the limited ABI symbols.
const WINDOWS_ABI3_LIB_NAME: &str = "python3";
const WINDOWS_ABI3_DEBUG_LIB_NAME: &str = "python3_d";

fn default_lib_name_for_target(
    version: PythonVersion,
    implementation: PythonImplementation,
    abi3: bool,
    gil_disabled: bool,
    target: &Triple,
) -> Option<String> {
    if target.operating_system == OperatingSystem::Windows {
        Some(
            default_lib_name_windows(version, implementation, abi3, false, false, gil_disabled)
                .unwrap(),
        )
    } else if is_linking_libpython_for_target(target) {
        Some(default_lib_name_unix(version, implementation, None, gil_disabled).unwrap())
    } else {
        None
    }
}

fn default_lib_name_windows(
    version: PythonVersion,
    implementation: PythonImplementation,
    abi3: bool,
    mingw: bool,
    debug: bool,
    gil_disabled: bool,
) -> Result<String> {
    if debug && version < PythonVersion::PY310 {
        // CPython bug: linking against python3_d.dll raises error
        // https://github.com/python/cpython/issues/101614
        Ok(format!("python{}{}_d", version.major, version.minor))
    } else if abi3 && !(gil_disabled || implementation.is_pypy() || implementation.is_graalpy()) {
        if debug {
            Ok(WINDOWS_ABI3_DEBUG_LIB_NAME.to_owned())
        } else {
            Ok(WINDOWS_ABI3_LIB_NAME.to_owned())
        }
    } else if mingw {
        ensure!(
            !gil_disabled,
            "MinGW free-threaded builds are not currently tested or supported"
        );
        // https://packages.msys2.org/base/mingw-w64-python
        Ok(format!("python{}.{}", version.major, version.minor))
    } else if gil_disabled {
        ensure!(version >= PythonVersion::PY313, "Cannot compile C extensions for the free-threaded build on Python versions earlier than 3.13, found {}.{}", version.major, version.minor);
        if debug {
            Ok(format!("python{}{}t_d", version.major, version.minor))
        } else {
            Ok(format!("python{}{}t", version.major, version.minor))
        }
    } else if debug {
        Ok(format!("python{}{}_d", version.major, version.minor))
    } else {
        Ok(format!("python{}{}", version.major, version.minor))
    }
}

fn default_lib_name_unix(
    version: PythonVersion,
    implementation: PythonImplementation,
    ld_version: Option<&str>,
    gil_disabled: bool,
) -> Result<String> {
    match implementation {
        PythonImplementation::CPython => match ld_version {
            Some(ld_version) => Ok(format!("python{ld_version}")),
            None => {
                if version > PythonVersion::PY37 {
                    // PEP 3149 ABI version tags are finally gone
                    if gil_disabled {
                        ensure!(version >= PythonVersion::PY313, "Cannot compile C extensions for the free-threaded build on Python versions earlier than 3.13, found {}.{}", version.major, version.minor);
                        Ok(format!("python{}.{}t", version.major, version.minor))
                    } else {
                        Ok(format!("python{}.{}", version.major, version.minor))
                    }
                } else {
                    // Work around https://bugs.python.org/issue36707
                    Ok(format!("python{}.{}m", version.major, version.minor))
                }
            }
        },
        PythonImplementation::PyPy => match ld_version {
            Some(ld_version) => Ok(format!("pypy{ld_version}-c")),
            None => Ok(format!("pypy{}.{}-c", version.major, version.minor)),
        },

        PythonImplementation::GraalPy => Ok("python-native".to_string()),
    }
}

/// Run a python script using the specified interpreter binary.
fn run_python_script(interpreter: &Path, script: &str) -> Result<String> {
    run_python_script_with_envs(interpreter, script, std::iter::empty::<(&str, &str)>())
}

/// Run a python script using the specified interpreter binary with additional environment
/// variables (e.g. PYTHONPATH) set.
fn run_python_script_with_envs<I, K, V>(interpreter: &Path, script: &str, envs: I) -> Result<String>
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    let out = Command::new(interpreter)
        .env("PYTHONIOENCODING", "utf-8")
        .envs(envs)
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

fn venv_interpreter(virtual_env: &OsStr, windows: bool) -> PathBuf {
    if windows {
        Path::new(virtual_env).join("Scripts").join("python.exe")
    } else {
        Path::new(virtual_env).join("bin").join("python")
    }
}

fn conda_env_interpreter(conda_prefix: &OsStr, windows: bool) -> PathBuf {
    if windows {
        Path::new(conda_prefix).join("python.exe")
    } else {
        Path::new(conda_prefix).join("bin").join("python")
    }
}

fn get_env_interpreter() -> Option<PathBuf> {
    match (env_var("VIRTUAL_ENV"), env_var("CONDA_PREFIX")) {
        // Use cfg rather than CARGO_CFG_TARGET_OS because this affects where files are located on the
        // build host
        (Some(dir), None) => Some(venv_interpreter(&dir, cfg!(windows))),
        (None, Some(dir)) => Some(conda_env_interpreter(&dir, cfg!(windows))),
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
    // Trigger rebuilds when `PYO3_ENVIRONMENT_SIGNATURE` env var value changes
    // See https://github.com/PyO3/pyo3/issues/2724
    println!("cargo:rerun-if-env-changed=PYO3_ENVIRONMENT_SIGNATURE");

    if let Some(exe) = env_var("PYO3_PYTHON") {
        Ok(exe.into())
    } else if let Some(env_interpreter) = get_env_interpreter() {
        Ok(env_interpreter)
    } else {
        println!("cargo:rerun-if-env-changed=PATH");
        ["python", "python3"]
            .iter()
            .find(|bin| {
                if let Ok(out) = Command::new(bin).arg("--version").output() {
                    // begin with `Python 3.X.X :: additional info`
                    out.stdout.starts_with(b"Python 3")
                        || out.stderr.starts_with(b"Python 3")
                        || out.stdout.starts_with(b"GraalPy 3")
                } else {
                    false
                }
            })
            .map(PathBuf::from)
            .ok_or_else(|| "no Python 3.x interpreter found".into())
    }
}

/// Locates and extracts the build host Python interpreter configuration.
///
/// Lowers the configured Python version to `abi3_version` if required.
fn get_host_interpreter(abi3_version: Option<PythonVersion>) -> Result<InterpreterConfig> {
    let interpreter_path = find_interpreter()?;

    let mut interpreter_config = InterpreterConfig::from_interpreter(interpreter_path)?;
    interpreter_config.fixup_for_abi3_version(abi3_version)?;

    Ok(interpreter_config)
}

/// Generates an interpreter config suitable for cross-compilation.
///
/// This must be called from PyO3's build script, because it relies on environment variables such as
/// CARGO_CFG_TARGET_OS which aren't available at any other time.
pub fn make_cross_compile_config() -> Result<Option<InterpreterConfig>> {
    let interpreter_config = if let Some(cross_config) = cross_compiling_from_cargo_env()? {
        let mut interpreter_config = load_cross_compile_config(cross_config)?;
        interpreter_config.fixup_for_abi3_version(get_abi3_version())?;
        Some(interpreter_config)
    } else {
        None
    };

    Ok(interpreter_config)
}

/// Generates an interpreter config which will be hard-coded into the pyo3-build-config crate.
/// Only used by `pyo3-build-config` build script.
#[allow(dead_code, unused_mut)]
pub fn make_interpreter_config() -> Result<InterpreterConfig> {
    let host = Triple::host();
    let abi3_version = get_abi3_version();

    // See if we can safely skip the Python interpreter configuration detection.
    // Unix "abi3" extension modules can usually be built without any interpreter.
    let need_interpreter = abi3_version.is_none() || require_libdir_for_target(&host);

    if have_python_interpreter() {
        match get_host_interpreter(abi3_version) {
            Ok(interpreter_config) => return Ok(interpreter_config),
            // Bail if the interpreter configuration is required to build.
            Err(e) if need_interpreter => return Err(e),
            _ => {
                // Fall back to the "abi3" defaults just as if `PYO3_NO_PYTHON`
                // environment variable was set.
                warn!("Compiling without a working Python interpreter.");
            }
        }
    } else {
        ensure!(
            abi3_version.is_some(),
            "An abi3-py3* feature must be specified when compiling without a Python interpreter."
        );
    };

    let mut interpreter_config = default_abi3_config(&host, abi3_version.unwrap())?;

    // Auto generate python3.dll import libraries for Windows targets.
    #[cfg(feature = "python3-dll-a")]
    {
        let gil_disabled = interpreter_config
            .build_flags
            .0
            .contains(&BuildFlag::Py_GIL_DISABLED);
        let py_version = if interpreter_config.implementation == PythonImplementation::CPython
            && interpreter_config.abi3
            && !gil_disabled
        {
            None
        } else {
            Some(interpreter_config.version)
        };
        interpreter_config.lib_dir = self::import_lib::generate_import_lib(
            &host,
            interpreter_config.implementation,
            py_version,
            None,
        )?;
    }

    Ok(interpreter_config)
}

fn escape(bytes: &[u8]) -> String {
    let mut escaped = String::with_capacity(2 * bytes.len());

    for byte in bytes {
        const LUT: &[u8; 16] = b"0123456789abcdef";

        escaped.push(LUT[(byte >> 4) as usize] as char);
        escaped.push(LUT[(byte & 0x0F) as usize] as char);
    }

    escaped
}

fn unescape(escaped: &str) -> Vec<u8> {
    assert!(escaped.len() % 2 == 0, "invalid hex encoding");

    let mut bytes = Vec::with_capacity(escaped.len() / 2);

    for chunk in escaped.as_bytes().chunks_exact(2) {
        fn unhex(hex: u8) -> u8 {
            match hex {
                b'a'..=b'f' => hex - b'a' + 10,
                b'0'..=b'9' => hex - b'0',
                _ => panic!("invalid hex encoding"),
            }
        }

        bytes.push((unhex(chunk[0]) << 4) | unhex(chunk[1]));
    }

    bytes
}

#[cfg(test)]
mod tests {
    use target_lexicon::triple;

    use super::*;

    #[test]
    fn test_config_file_roundtrip() {
        let config = InterpreterConfig {
            abi3: true,
            build_flags: BuildFlags::default(),
            pointer_width: Some(32),
            executable: Some("executable".into()),
            implementation: PythonImplementation::CPython,
            lib_name: Some("lib_name".into()),
            lib_dir: Some("lib_dir".into()),
            shared: true,
            version: MINIMUM_SUPPORTED_VERSION,
            suppress_build_script_link_lines: true,
            extra_build_script_lines: vec!["cargo:test1".to_string(), "cargo:test2".to_string()],
            python_framework_prefix: None,
        };
        let mut buf: Vec<u8> = Vec::new();
        config.to_writer(&mut buf).unwrap();

        assert_eq!(config, InterpreterConfig::from_reader(&*buf).unwrap());

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
            suppress_build_script_link_lines: false,
            extra_build_script_lines: vec![],
            python_framework_prefix: None,
        };
        let mut buf: Vec<u8> = Vec::new();
        config.to_writer(&mut buf).unwrap();

        assert_eq!(config, InterpreterConfig::from_reader(&*buf).unwrap());
    }

    #[test]
    fn test_config_file_roundtrip_with_escaping() {
        let config = InterpreterConfig {
            abi3: true,
            build_flags: BuildFlags::default(),
            pointer_width: Some(32),
            executable: Some("executable".into()),
            implementation: PythonImplementation::CPython,
            lib_name: Some("lib_name".into()),
            lib_dir: Some("lib_dir\\n".into()),
            shared: true,
            version: MINIMUM_SUPPORTED_VERSION,
            suppress_build_script_link_lines: true,
            extra_build_script_lines: vec!["cargo:test1".to_string(), "cargo:test2".to_string()],
            python_framework_prefix: None,
        };
        let mut buf: Vec<u8> = Vec::new();
        config.to_writer(&mut buf).unwrap();

        let buf = unescape(&escape(&buf));

        assert_eq!(config, InterpreterConfig::from_reader(&*buf).unwrap());
    }

    #[test]
    fn test_config_file_defaults() {
        // Only version is required
        assert_eq!(
            InterpreterConfig::from_reader("version=3.7".as_bytes()).unwrap(),
            InterpreterConfig {
                version: PythonVersion { major: 3, minor: 7 },
                implementation: PythonImplementation::CPython,
                shared: true,
                abi3: false,
                lib_name: None,
                lib_dir: None,
                executable: None,
                pointer_width: None,
                build_flags: BuildFlags::default(),
                suppress_build_script_link_lines: false,
                extra_build_script_lines: vec![],
                python_framework_prefix: None,
            }
        )
    }

    #[test]
    fn test_config_file_unknown_keys() {
        // ext_suffix is unknown to pyo3-build-config, but it shouldn't error
        assert_eq!(
            InterpreterConfig::from_reader("version=3.7\next_suffix=.python37.so".as_bytes())
                .unwrap(),
            InterpreterConfig {
                version: PythonVersion { major: 3, minor: 7 },
                implementation: PythonImplementation::CPython,
                shared: true,
                abi3: false,
                lib_name: None,
                lib_dir: None,
                executable: None,
                pointer_width: None,
                build_flags: BuildFlags::default(),
                suppress_build_script_link_lines: false,
                extra_build_script_lines: vec![],
                python_framework_prefix: None,
            }
        )
    }

    #[test]
    fn build_flags_default() {
        assert_eq!(BuildFlags::default(), BuildFlags::new());
    }

    #[test]
    fn build_flags_from_sysconfigdata() {
        let mut sysconfigdata = Sysconfigdata::new();

        assert_eq!(
            BuildFlags::from_sysconfigdata(&sysconfigdata).0,
            HashSet::new()
        );

        for flag in &BuildFlags::ALL {
            sysconfigdata.insert(flag.to_string(), "0".into());
        }

        assert_eq!(
            BuildFlags::from_sysconfigdata(&sysconfigdata).0,
            HashSet::new()
        );

        let mut expected_flags = HashSet::new();
        for flag in &BuildFlags::ALL {
            sysconfigdata.insert(flag.to_string(), "1".into());
            expected_flags.insert(flag.clone());
        }

        assert_eq!(
            BuildFlags::from_sysconfigdata(&sysconfigdata).0,
            expected_flags
        );
    }

    #[test]
    fn build_flags_fixup() {
        let mut build_flags = BuildFlags::new();

        build_flags = build_flags.fixup();
        assert!(build_flags.0.is_empty());

        build_flags.0.insert(BuildFlag::Py_DEBUG);

        build_flags = build_flags.fixup();

        // Py_DEBUG implies Py_REF_DEBUG
        assert!(build_flags.0.contains(&BuildFlag::Py_REF_DEBUG));
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
    fn config_from_empty_sysconfigdata() {
        let sysconfigdata = Sysconfigdata::new();
        assert!(InterpreterConfig::from_sysconfigdata(&sysconfigdata).is_err());
    }

    #[test]
    fn config_from_sysconfigdata() {
        let mut sysconfigdata = Sysconfigdata::new();
        // these are the minimal values required such that InterpreterConfig::from_sysconfigdata
        // does not error
        sysconfigdata.insert("SOABI", "cpython-37m-x86_64-linux-gnu");
        sysconfigdata.insert("VERSION", "3.7");
        sysconfigdata.insert("Py_ENABLE_SHARED", "1");
        sysconfigdata.insert("LIBDIR", "/usr/lib");
        sysconfigdata.insert("LDVERSION", "3.7m");
        sysconfigdata.insert("SIZEOF_VOID_P", "8");
        assert_eq!(
            InterpreterConfig::from_sysconfigdata(&sysconfigdata).unwrap(),
            InterpreterConfig {
                abi3: false,
                build_flags: BuildFlags::from_sysconfigdata(&sysconfigdata),
                pointer_width: Some(64),
                executable: None,
                implementation: PythonImplementation::CPython,
                lib_dir: Some("/usr/lib".into()),
                lib_name: Some("python3.7m".into()),
                shared: true,
                version: PythonVersion::PY37,
                suppress_build_script_link_lines: false,
                extra_build_script_lines: vec![],
                python_framework_prefix: None,
            }
        );
    }

    #[test]
    fn config_from_sysconfigdata_framework() {
        let mut sysconfigdata = Sysconfigdata::new();
        sysconfigdata.insert("SOABI", "cpython-37m-x86_64-linux-gnu");
        sysconfigdata.insert("VERSION", "3.7");
        // PYTHONFRAMEWORK should override Py_ENABLE_SHARED
        sysconfigdata.insert("Py_ENABLE_SHARED", "0");
        sysconfigdata.insert("PYTHONFRAMEWORK", "Python");
        sysconfigdata.insert("LIBDIR", "/usr/lib");
        sysconfigdata.insert("LDVERSION", "3.7m");
        sysconfigdata.insert("SIZEOF_VOID_P", "8");
        assert_eq!(
            InterpreterConfig::from_sysconfigdata(&sysconfigdata).unwrap(),
            InterpreterConfig {
                abi3: false,
                build_flags: BuildFlags::from_sysconfigdata(&sysconfigdata),
                pointer_width: Some(64),
                executable: None,
                implementation: PythonImplementation::CPython,
                lib_dir: Some("/usr/lib".into()),
                lib_name: Some("python3.7m".into()),
                shared: true,
                version: PythonVersion::PY37,
                suppress_build_script_link_lines: false,
                extra_build_script_lines: vec![],
                python_framework_prefix: None,
            }
        );

        sysconfigdata = Sysconfigdata::new();
        sysconfigdata.insert("SOABI", "cpython-37m-x86_64-linux-gnu");
        sysconfigdata.insert("VERSION", "3.7");
        // An empty PYTHONFRAMEWORK means it is not a framework
        sysconfigdata.insert("Py_ENABLE_SHARED", "0");
        sysconfigdata.insert("PYTHONFRAMEWORK", "");
        sysconfigdata.insert("LIBDIR", "/usr/lib");
        sysconfigdata.insert("LDVERSION", "3.7m");
        sysconfigdata.insert("SIZEOF_VOID_P", "8");
        assert_eq!(
            InterpreterConfig::from_sysconfigdata(&sysconfigdata).unwrap(),
            InterpreterConfig {
                abi3: false,
                build_flags: BuildFlags::from_sysconfigdata(&sysconfigdata),
                pointer_width: Some(64),
                executable: None,
                implementation: PythonImplementation::CPython,
                lib_dir: Some("/usr/lib".into()),
                lib_name: Some("python3.7m".into()),
                shared: false,
                version: PythonVersion::PY37,
                suppress_build_script_link_lines: false,
                extra_build_script_lines: vec![],
                python_framework_prefix: None,
            }
        );
    }

    #[test]
    fn windows_hardcoded_abi3_compile() {
        let host = triple!("x86_64-pc-windows-msvc");
        let min_version = "3.7".parse().unwrap();

        assert_eq!(
            default_abi3_config(&host, min_version).unwrap(),
            InterpreterConfig {
                implementation: PythonImplementation::CPython,
                version: PythonVersion { major: 3, minor: 7 },
                shared: true,
                abi3: true,
                lib_name: Some("python3".into()),
                lib_dir: None,
                executable: None,
                pointer_width: None,
                build_flags: BuildFlags::default(),
                suppress_build_script_link_lines: false,
                extra_build_script_lines: vec![],
                python_framework_prefix: None,
            }
        );
    }

    #[test]
    fn unix_hardcoded_abi3_compile() {
        let host = triple!("x86_64-unknown-linux-gnu");
        let min_version = "3.9".parse().unwrap();

        assert_eq!(
            default_abi3_config(&host, min_version).unwrap(),
            InterpreterConfig {
                implementation: PythonImplementation::CPython,
                version: PythonVersion { major: 3, minor: 9 },
                shared: true,
                abi3: true,
                lib_name: None,
                lib_dir: None,
                executable: None,
                pointer_width: None,
                build_flags: BuildFlags::default(),
                suppress_build_script_link_lines: false,
                extra_build_script_lines: vec![],
                python_framework_prefix: None,
            }
        );
    }

    #[test]
    fn windows_hardcoded_cross_compile() {
        let env_vars = CrossCompileEnvVars {
            pyo3_cross: None,
            pyo3_cross_lib_dir: Some("C:\\some\\path".into()),
            pyo3_cross_python_implementation: None,
            pyo3_cross_python_version: Some("3.7".into()),
        };

        let host = triple!("x86_64-unknown-linux-gnu");
        let target = triple!("i686-pc-windows-msvc");
        let cross_config =
            CrossCompileConfig::try_from_env_vars_host_target(env_vars, &host, &target)
                .unwrap()
                .unwrap();

        assert_eq!(
            default_cross_compile(&cross_config).unwrap(),
            InterpreterConfig {
                implementation: PythonImplementation::CPython,
                version: PythonVersion { major: 3, minor: 7 },
                shared: true,
                abi3: false,
                lib_name: Some("python37".into()),
                lib_dir: Some("C:\\some\\path".into()),
                executable: None,
                pointer_width: None,
                build_flags: BuildFlags::default(),
                suppress_build_script_link_lines: false,
                extra_build_script_lines: vec![],
                python_framework_prefix: None,
            }
        );
    }

    #[test]
    fn mingw_hardcoded_cross_compile() {
        let env_vars = CrossCompileEnvVars {
            pyo3_cross: None,
            pyo3_cross_lib_dir: Some("/usr/lib/mingw".into()),
            pyo3_cross_python_implementation: None,
            pyo3_cross_python_version: Some("3.8".into()),
        };

        let host = triple!("x86_64-unknown-linux-gnu");
        let target = triple!("i686-pc-windows-gnu");
        let cross_config =
            CrossCompileConfig::try_from_env_vars_host_target(env_vars, &host, &target)
                .unwrap()
                .unwrap();

        assert_eq!(
            default_cross_compile(&cross_config).unwrap(),
            InterpreterConfig {
                implementation: PythonImplementation::CPython,
                version: PythonVersion { major: 3, minor: 8 },
                shared: true,
                abi3: false,
                lib_name: Some("python38".into()),
                lib_dir: Some("/usr/lib/mingw".into()),
                executable: None,
                pointer_width: None,
                build_flags: BuildFlags::default(),
                suppress_build_script_link_lines: false,
                extra_build_script_lines: vec![],
                python_framework_prefix: None,
            }
        );
    }

    #[test]
    fn unix_hardcoded_cross_compile() {
        let env_vars = CrossCompileEnvVars {
            pyo3_cross: None,
            pyo3_cross_lib_dir: Some("/usr/arm64/lib".into()),
            pyo3_cross_python_implementation: None,
            pyo3_cross_python_version: Some("3.9".into()),
        };

        let host = triple!("x86_64-unknown-linux-gnu");
        let target = triple!("aarch64-unknown-linux-gnu");
        let cross_config =
            CrossCompileConfig::try_from_env_vars_host_target(env_vars, &host, &target)
                .unwrap()
                .unwrap();

        assert_eq!(
            default_cross_compile(&cross_config).unwrap(),
            InterpreterConfig {
                implementation: PythonImplementation::CPython,
                version: PythonVersion { major: 3, minor: 9 },
                shared: true,
                abi3: false,
                lib_name: Some("python3.9".into()),
                lib_dir: Some("/usr/arm64/lib".into()),
                executable: None,
                pointer_width: None,
                build_flags: BuildFlags::default(),
                suppress_build_script_link_lines: false,
                extra_build_script_lines: vec![],
                python_framework_prefix: None,
            }
        );
    }

    #[test]
    fn pypy_hardcoded_cross_compile() {
        let env_vars = CrossCompileEnvVars {
            pyo3_cross: None,
            pyo3_cross_lib_dir: None,
            pyo3_cross_python_implementation: Some("PyPy".into()),
            pyo3_cross_python_version: Some("3.10".into()),
        };

        let triple = triple!("x86_64-unknown-linux-gnu");
        let cross_config =
            CrossCompileConfig::try_from_env_vars_host_target(env_vars, &triple, &triple)
                .unwrap()
                .unwrap();

        assert_eq!(
            default_cross_compile(&cross_config).unwrap(),
            InterpreterConfig {
                implementation: PythonImplementation::PyPy,
                version: PythonVersion {
                    major: 3,
                    minor: 10
                },
                shared: true,
                abi3: false,
                lib_name: Some("pypy3.10-c".into()),
                lib_dir: None,
                executable: None,
                pointer_width: None,
                build_flags: BuildFlags::default(),
                suppress_build_script_link_lines: false,
                extra_build_script_lines: vec![],
                python_framework_prefix: None,
            }
        );
    }

    #[test]
    fn default_lib_name_windows() {
        use PythonImplementation::*;
        assert_eq!(
            super::default_lib_name_windows(
                PythonVersion { major: 3, minor: 9 },
                CPython,
                false,
                false,
                false,
                false,
            )
            .unwrap(),
            "python39",
        );
        assert!(super::default_lib_name_windows(
            PythonVersion { major: 3, minor: 9 },
            CPython,
            false,
            false,
            false,
            true,
        )
        .is_err());
        assert_eq!(
            super::default_lib_name_windows(
                PythonVersion { major: 3, minor: 9 },
                CPython,
                true,
                false,
                false,
                false,
            )
            .unwrap(),
            "python3",
        );
        assert_eq!(
            super::default_lib_name_windows(
                PythonVersion { major: 3, minor: 9 },
                CPython,
                false,
                true,
                false,
                false,
            )
            .unwrap(),
            "python3.9",
        );
        assert_eq!(
            super::default_lib_name_windows(
                PythonVersion { major: 3, minor: 9 },
                CPython,
                true,
                true,
                false,
                false,
            )
            .unwrap(),
            "python3",
        );
        assert_eq!(
            super::default_lib_name_windows(
                PythonVersion { major: 3, minor: 9 },
                PyPy,
                true,
                false,
                false,
                false,
            )
            .unwrap(),
            "python39",
        );
        assert_eq!(
            super::default_lib_name_windows(
                PythonVersion { major: 3, minor: 9 },
                CPython,
                false,
                false,
                true,
                false,
            )
            .unwrap(),
            "python39_d",
        );
        // abi3 debug builds on windows use version-specific lib on 3.9 and older
        // to workaround https://github.com/python/cpython/issues/101614
        assert_eq!(
            super::default_lib_name_windows(
                PythonVersion { major: 3, minor: 9 },
                CPython,
                true,
                false,
                true,
                false,
            )
            .unwrap(),
            "python39_d",
        );
        assert_eq!(
            super::default_lib_name_windows(
                PythonVersion {
                    major: 3,
                    minor: 10
                },
                CPython,
                true,
                false,
                true,
                false,
            )
            .unwrap(),
            "python3_d",
        );
        // Python versions older than 3.13 don't support gil_disabled
        assert!(super::default_lib_name_windows(
            PythonVersion {
                major: 3,
                minor: 12,
            },
            CPython,
            false,
            false,
            false,
            true,
        )
        .is_err());
        // mingw and free-threading are incompatible (until someone adds support)
        assert!(super::default_lib_name_windows(
            PythonVersion {
                major: 3,
                minor: 12,
            },
            CPython,
            false,
            true,
            false,
            true,
        )
        .is_err());
        assert_eq!(
            super::default_lib_name_windows(
                PythonVersion {
                    major: 3,
                    minor: 13
                },
                CPython,
                false,
                false,
                false,
                true,
            )
            .unwrap(),
            "python313t",
        );
        assert_eq!(
            super::default_lib_name_windows(
                PythonVersion {
                    major: 3,
                    minor: 13
                },
                CPython,
                true, // abi3 true should not affect the free-threaded lib name
                false,
                false,
                true,
            )
            .unwrap(),
            "python313t",
        );
        assert_eq!(
            super::default_lib_name_windows(
                PythonVersion {
                    major: 3,
                    minor: 13
                },
                CPython,
                false,
                false,
                true,
                true,
            )
            .unwrap(),
            "python313t_d",
        );
    }

    #[test]
    fn default_lib_name_unix() {
        use PythonImplementation::*;
        // Defaults to python3.7m for CPython 3.7
        assert_eq!(
            super::default_lib_name_unix(
                PythonVersion { major: 3, minor: 7 },
                CPython,
                None,
                false
            )
            .unwrap(),
            "python3.7m",
        );
        // Defaults to pythonX.Y for CPython 3.8+
        assert_eq!(
            super::default_lib_name_unix(
                PythonVersion { major: 3, minor: 8 },
                CPython,
                None,
                false
            )
            .unwrap(),
            "python3.8",
        );
        assert_eq!(
            super::default_lib_name_unix(
                PythonVersion { major: 3, minor: 9 },
                CPython,
                None,
                false
            )
            .unwrap(),
            "python3.9",
        );
        // Can use ldversion to override for CPython
        assert_eq!(
            super::default_lib_name_unix(
                PythonVersion { major: 3, minor: 9 },
                CPython,
                Some("3.7md"),
                false
            )
            .unwrap(),
            "python3.7md",
        );

        // PyPy 3.9 includes ldversion
        assert_eq!(
            super::default_lib_name_unix(PythonVersion { major: 3, minor: 9 }, PyPy, None, false)
                .unwrap(),
            "pypy3.9-c",
        );

        assert_eq!(
            super::default_lib_name_unix(
                PythonVersion { major: 3, minor: 9 },
                PyPy,
                Some("3.9d"),
                false
            )
            .unwrap(),
            "pypy3.9d-c",
        );

        // free-threading adds a t suffix
        assert_eq!(
            super::default_lib_name_unix(
                PythonVersion {
                    major: 3,
                    minor: 13
                },
                CPython,
                None,
                true
            )
            .unwrap(),
            "python3.13t",
        );
        // 3.12 and older are incompatible with gil_disabled
        assert!(super::default_lib_name_unix(
            PythonVersion {
                major: 3,
                minor: 12,
            },
            CPython,
            None,
            true,
        )
        .is_err());
    }

    #[test]
    fn parse_cross_python_version() {
        let env_vars = CrossCompileEnvVars {
            pyo3_cross: None,
            pyo3_cross_lib_dir: None,
            pyo3_cross_python_version: Some("3.9".into()),
            pyo3_cross_python_implementation: None,
        };

        assert_eq!(
            env_vars.parse_version().unwrap(),
            (Some(PythonVersion { major: 3, minor: 9 }), None),
        );

        let env_vars = CrossCompileEnvVars {
            pyo3_cross: None,
            pyo3_cross_lib_dir: None,
            pyo3_cross_python_version: None,
            pyo3_cross_python_implementation: None,
        };

        assert_eq!(env_vars.parse_version().unwrap(), (None, None));

        let env_vars = CrossCompileEnvVars {
            pyo3_cross: None,
            pyo3_cross_lib_dir: None,
            pyo3_cross_python_version: Some("3.13t".into()),
            pyo3_cross_python_implementation: None,
        };

        assert_eq!(
            env_vars.parse_version().unwrap(),
            (
                Some(PythonVersion {
                    major: 3,
                    minor: 13
                }),
                Some("t".into())
            ),
        );

        let env_vars = CrossCompileEnvVars {
            pyo3_cross: None,
            pyo3_cross_lib_dir: None,
            pyo3_cross_python_version: Some("100".into()),
            pyo3_cross_python_implementation: None,
        };

        assert!(env_vars.parse_version().is_err());
    }

    #[test]
    fn interpreter_version_reduced_to_abi3() {
        let mut config = InterpreterConfig {
            abi3: true,
            build_flags: BuildFlags::default(),
            pointer_width: None,
            executable: None,
            implementation: PythonImplementation::CPython,
            lib_dir: None,
            lib_name: None,
            shared: true,
            version: PythonVersion { major: 3, minor: 7 },
            suppress_build_script_link_lines: false,
            extra_build_script_lines: vec![],
            python_framework_prefix: None,
        };

        config
            .fixup_for_abi3_version(Some(PythonVersion { major: 3, minor: 7 }))
            .unwrap();
        assert_eq!(config.version, PythonVersion { major: 3, minor: 7 });
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
            version: PythonVersion { major: 3, minor: 7 },
            suppress_build_script_link_lines: false,
            extra_build_script_lines: vec![],
            python_framework_prefix: None,
        };

        assert!(config
            .fixup_for_abi3_version(Some(PythonVersion { major: 3, minor: 8 }))
            .unwrap_err()
            .to_string()
            .contains(
                "cannot set a minimum Python version 3.8 higher than the interpreter version 3.7"
            ));
    }

    #[test]
    #[cfg(all(
        target_os = "linux",
        target_arch = "x86_64",
        feature = "resolve-config"
    ))]
    fn parse_sysconfigdata() {
        // A best effort attempt to get test coverage for the sysconfigdata parsing.
        // Might not complete successfully depending on host installation; that's ok as long as
        // CI demonstrates this path is covered!

        let interpreter_config = crate::get();

        let lib_dir = match &interpreter_config.lib_dir {
            Some(lib_dir) => Path::new(lib_dir),
            // Don't know where to search for sysconfigdata; never mind.
            None => return,
        };

        let cross = CrossCompileConfig {
            lib_dir: Some(lib_dir.into()),
            version: Some(interpreter_config.version),
            implementation: Some(interpreter_config.implementation),
            target: triple!("x86_64-unknown-linux-gnu"),
            abiflags: if interpreter_config.is_free_threaded() {
                Some("t".into())
            } else {
                None
            },
        };

        let sysconfigdata_path = match find_sysconfigdata(&cross) {
            Ok(Some(path)) => path,
            // Couldn't find a matching sysconfigdata; never mind!
            _ => return,
        };
        let sysconfigdata = super::parse_sysconfigdata(sysconfigdata_path).unwrap();
        let parsed_config = InterpreterConfig::from_sysconfigdata(&sysconfigdata).unwrap();

        assert_eq!(
            parsed_config,
            InterpreterConfig {
                abi3: false,
                build_flags: BuildFlags(interpreter_config.build_flags.0.clone()),
                pointer_width: Some(64),
                executable: None,
                implementation: PythonImplementation::CPython,
                lib_dir: interpreter_config.lib_dir.to_owned(),
                lib_name: interpreter_config.lib_name.to_owned(),
                shared: true,
                version: interpreter_config.version,
                suppress_build_script_link_lines: false,
                extra_build_script_lines: vec![],
                python_framework_prefix: None,
            }
        )
    }

    #[test]
    fn test_venv_interpreter() {
        let base = OsStr::new("base");
        assert_eq!(
            venv_interpreter(base, true),
            PathBuf::from_iter(&["base", "Scripts", "python.exe"])
        );
        assert_eq!(
            venv_interpreter(base, false),
            PathBuf::from_iter(&["base", "bin", "python"])
        );
    }

    #[test]
    fn test_conda_env_interpreter() {
        let base = OsStr::new("base");
        assert_eq!(
            conda_env_interpreter(base, true),
            PathBuf::from_iter(&["base", "python.exe"])
        );
        assert_eq!(
            conda_env_interpreter(base, false),
            PathBuf::from_iter(&["base", "bin", "python"])
        );
    }

    #[test]
    fn test_not_cross_compiling_from_to() {
        assert!(cross_compiling_from_to(
            &triple!("x86_64-unknown-linux-gnu"),
            &triple!("x86_64-unknown-linux-gnu"),
        )
        .unwrap()
        .is_none());

        assert!(cross_compiling_from_to(
            &triple!("x86_64-apple-darwin"),
            &triple!("x86_64-apple-darwin")
        )
        .unwrap()
        .is_none());

        assert!(cross_compiling_from_to(
            &triple!("aarch64-apple-darwin"),
            &triple!("x86_64-apple-darwin")
        )
        .unwrap()
        .is_none());

        assert!(cross_compiling_from_to(
            &triple!("x86_64-apple-darwin"),
            &triple!("aarch64-apple-darwin")
        )
        .unwrap()
        .is_none());

        assert!(cross_compiling_from_to(
            &triple!("x86_64-pc-windows-msvc"),
            &triple!("i686-pc-windows-msvc")
        )
        .unwrap()
        .is_none());

        assert!(cross_compiling_from_to(
            &triple!("x86_64-unknown-linux-gnu"),
            &triple!("x86_64-unknown-linux-musl")
        )
        .unwrap()
        .is_none());

        assert!(cross_compiling_from_to(
            &triple!("x86_64-pc-windows-msvc"),
            &triple!("x86_64-win7-windows-msvc"),
        )
        .unwrap()
        .is_none());
    }

    #[test]
    fn test_is_cross_compiling_from_to() {
        assert!(cross_compiling_from_to(
            &triple!("x86_64-pc-windows-msvc"),
            &triple!("aarch64-pc-windows-msvc")
        )
        .unwrap()
        .is_some());
    }

    #[test]
    fn test_run_python_script() {
        // as above, this should be okay in CI where Python is presumed installed
        let interpreter = make_interpreter_config()
            .expect("could not get InterpreterConfig from installed interpreter");
        let out = interpreter
            .run_python_script("print(2 + 2)")
            .expect("failed to run Python script");
        assert_eq!(out.trim_end(), "4");
    }

    #[test]
    fn test_run_python_script_with_envs() {
        // as above, this should be okay in CI where Python is presumed installed
        let interpreter = make_interpreter_config()
            .expect("could not get InterpreterConfig from installed interpreter");
        let out = interpreter
            .run_python_script_with_envs(
                "import os; print(os.getenv('PYO3_TEST'))",
                vec![("PYO3_TEST", "42")],
            )
            .expect("failed to run Python script");
        assert_eq!(out.trim_end(), "42");
    }

    #[test]
    fn test_build_script_outputs_base() {
        let interpreter_config = InterpreterConfig {
            implementation: PythonImplementation::CPython,
            version: PythonVersion { major: 3, minor: 9 },
            shared: true,
            abi3: false,
            lib_name: Some("python3".into()),
            lib_dir: None,
            executable: None,
            pointer_width: None,
            build_flags: BuildFlags::default(),
            suppress_build_script_link_lines: false,
            extra_build_script_lines: vec![],
            python_framework_prefix: None,
        };
        assert_eq!(
            interpreter_config.build_script_outputs(),
            [
                "cargo:rustc-cfg=Py_3_7".to_owned(),
                "cargo:rustc-cfg=Py_3_8".to_owned(),
                "cargo:rustc-cfg=Py_3_9".to_owned(),
            ]
        );

        let interpreter_config = InterpreterConfig {
            implementation: PythonImplementation::PyPy,
            ..interpreter_config
        };
        assert_eq!(
            interpreter_config.build_script_outputs(),
            [
                "cargo:rustc-cfg=Py_3_7".to_owned(),
                "cargo:rustc-cfg=Py_3_8".to_owned(),
                "cargo:rustc-cfg=Py_3_9".to_owned(),
                "cargo:rustc-cfg=PyPy".to_owned(),
            ]
        );
    }

    #[test]
    fn test_build_script_outputs_abi3() {
        let interpreter_config = InterpreterConfig {
            implementation: PythonImplementation::CPython,
            version: PythonVersion { major: 3, minor: 9 },
            shared: true,
            abi3: true,
            lib_name: Some("python3".into()),
            lib_dir: None,
            executable: None,
            pointer_width: None,
            build_flags: BuildFlags::default(),
            suppress_build_script_link_lines: false,
            extra_build_script_lines: vec![],
            python_framework_prefix: None,
        };

        assert_eq!(
            interpreter_config.build_script_outputs(),
            [
                "cargo:rustc-cfg=Py_3_7".to_owned(),
                "cargo:rustc-cfg=Py_3_8".to_owned(),
                "cargo:rustc-cfg=Py_3_9".to_owned(),
                "cargo:rustc-cfg=Py_LIMITED_API".to_owned(),
            ]
        );

        let interpreter_config = InterpreterConfig {
            implementation: PythonImplementation::PyPy,
            ..interpreter_config
        };
        assert_eq!(
            interpreter_config.build_script_outputs(),
            [
                "cargo:rustc-cfg=Py_3_7".to_owned(),
                "cargo:rustc-cfg=Py_3_8".to_owned(),
                "cargo:rustc-cfg=Py_3_9".to_owned(),
                "cargo:rustc-cfg=PyPy".to_owned(),
                "cargo:rustc-cfg=Py_LIMITED_API".to_owned(),
            ]
        );
    }

    #[test]
    fn test_build_script_outputs_gil_disabled() {
        let mut build_flags = BuildFlags::default();
        build_flags.0.insert(BuildFlag::Py_GIL_DISABLED);
        let interpreter_config = InterpreterConfig {
            implementation: PythonImplementation::CPython,
            version: PythonVersion {
                major: 3,
                minor: 13,
            },
            shared: true,
            abi3: false,
            lib_name: Some("python3".into()),
            lib_dir: None,
            executable: None,
            pointer_width: None,
            build_flags,
            suppress_build_script_link_lines: false,
            extra_build_script_lines: vec![],
            python_framework_prefix: None,
        };

        assert_eq!(
            interpreter_config.build_script_outputs(),
            [
                "cargo:rustc-cfg=Py_3_7".to_owned(),
                "cargo:rustc-cfg=Py_3_8".to_owned(),
                "cargo:rustc-cfg=Py_3_9".to_owned(),
                "cargo:rustc-cfg=Py_3_10".to_owned(),
                "cargo:rustc-cfg=Py_3_11".to_owned(),
                "cargo:rustc-cfg=Py_3_12".to_owned(),
                "cargo:rustc-cfg=Py_3_13".to_owned(),
                "cargo:rustc-cfg=Py_GIL_DISABLED".to_owned(),
            ]
        );
    }

    #[test]
    fn test_build_script_outputs_debug() {
        let mut build_flags = BuildFlags::default();
        build_flags.0.insert(BuildFlag::Py_DEBUG);
        let interpreter_config = InterpreterConfig {
            implementation: PythonImplementation::CPython,
            version: PythonVersion { major: 3, minor: 7 },
            shared: true,
            abi3: false,
            lib_name: Some("python3".into()),
            lib_dir: None,
            executable: None,
            pointer_width: None,
            build_flags,
            suppress_build_script_link_lines: false,
            extra_build_script_lines: vec![],
            python_framework_prefix: None,
        };

        assert_eq!(
            interpreter_config.build_script_outputs(),
            [
                "cargo:rustc-cfg=Py_3_7".to_owned(),
                "cargo:rustc-cfg=py_sys_config=\"Py_DEBUG\"".to_owned(),
            ]
        );
    }

    #[test]
    fn test_find_sysconfigdata_in_invalid_lib_dir() {
        let e = find_all_sysconfigdata(&CrossCompileConfig {
            lib_dir: Some(PathBuf::from("/abc/123/not/a/real/path")),
            version: None,
            implementation: None,
            target: triple!("x86_64-unknown-linux-gnu"),
            abiflags: None,
        })
        .unwrap_err();

        // actual error message is platform-dependent, so just check the context we add
        assert!(e.report().to_string().starts_with(
            "failed to search the lib dir at 'PYO3_CROSS_LIB_DIR=/abc/123/not/a/real/path'\n\
            caused by:\n  \
              - 0: failed to list the entries in '/abc/123/not/a/real/path'\n  \
              - 1: \
            "
        ));
    }

    #[test]
    fn test_from_pyo3_config_file_env_rebuild() {
        READ_ENV_VARS.with(|vars| vars.borrow_mut().clear());
        let _ = InterpreterConfig::from_pyo3_config_file_env();
        // it's possible that other env vars were also read, hence just checking for contains
        READ_ENV_VARS.with(|vars| assert!(vars.borrow().contains(&"PYO3_CONFIG_FILE".to_string())));
    }
}
