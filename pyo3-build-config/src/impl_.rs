//! Main implementation module included in both the `pyo3-build-config` library crate
//! and its build script.

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
pub(crate) const MINIMUM_SUPPORTED_VERSION: PythonVersion = PythonVersion { major: 3, minor: 8 };

pub(crate) const MINIMUM_SUPPORTED_VERSION_PYPY: PythonVersion = PythonVersion {
    major: 3,
    minor: 11,
};
pub(crate) const MAXIMUM_SUPPORTED_VERSION_PYPY: PythonVersion = PythonVersion {
    major: 3,
    minor: 11,
};

/// GraalPy may implement the same CPython version over multiple releases.
const MINIMUM_SUPPORTED_VERSION_GRAALPY: PythonVersion = PythonVersion {
    major: 25,
    minor: 0,
};

/// Maximum Python version that can be used as minimum required Python version with abi3.
pub(crate) const STABLE_ABI_MAX_MINOR: u8 = 14;

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
    /// The host Python implementation flavor.
    ///
    /// Serialized to `implementation`.
    pub implementation: PythonImplementation,

    /// The host Python `X.Y` version. e.g. `3.9`.
    ///
    /// Serialized to `version`.
    pub version: PythonVersion,

    /// Whether link library is shared.
    ///
    /// Serialized to `shared`.
    pub shared: bool,

    /// The ABI to use for the compilation target.
    ///
    /// Serialized to `target_abi`.
    /// See the documentation for the PythonAbi enum for more details.
    pub target_abi: PythonAbi,

    /// Deprecated field used to indicate an abi3 target.
    ///
    /// Creating an InterpreterConfig struct with `abi3` set to `True`,
    /// `interpreter` set to `PythonImplementation::CPython` and `version` set
    /// to `PythonVersion {major: 3, minor: 9}` is equivalent to setting `abi`
    /// to `PythonAbiBuilder::new(PythonImplementation::CPython, PythonVersion
    /// {major: 3, minor: 9).abi3().finalize()`.
    ///
    /// Serialized to `abi3`.
    #[deprecated]
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
        assert!(self.target_abi.version >= MINIMUM_SUPPORTED_VERSION);

        let mut out = vec![];

        for i in MINIMUM_SUPPORTED_VERSION.minor..=self.target_abi.version.minor {
            out.push(format!("cargo:rustc-cfg=Py_3_{i}"));
        }

        match self.target_abi.implementation {
            PythonImplementation::CPython => {}
            PythonImplementation::PyPy => out.push("cargo:rustc-cfg=PyPy".to_owned()),
            PythonImplementation::GraalPy => out.push("cargo:rustc-cfg=GraalPy".to_owned()),
            PythonImplementation::RustPython => out.push("cargo:rustc-cfg=RustPython".to_owned()),
        }

        match self.target_abi.kind {
            PythonAbiKind::Abi3 => {
                if !self.target_abi.kind.is_free_threaded() {
                    out.push("cargo:rustc-cfg=Py_LIMITED_API".to_owned());
                }
            }
            PythonAbiKind::VersionSpecific(_) => {}
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
    pub fn from_interpreter(
        interpreter: impl AsRef<Path>,
        abi3_version: Option<PythonVersion>,
    ) -> Result<Self> {
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
print("cygwin", get_platform().startswith("cygwin"))
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

        let implementation = map["implementation"].parse()?;

        let gil_disabled = match map["gil_disabled"].as_str() {
            "1" => true,
            "0" => false,
            "None" => false,
            _ => panic!("Unknown Py_GIL_DISABLED value"),
        };

        let target_version = if let Some(min_version) = abi3_version {
            ensure!(
                min_version <= version,
                "cannot set a minimum Python version {} higher than the interpreter version {} \
                 (the minimum Python version is implied by the abi3-py3{} feature)",
                min_version,
                version,
                min_version.minor
            );
            min_version
        } else {
            version
        };

        let mut abi_builder = PythonAbiBuilder::new(implementation, target_version);

        if gil_disabled {
            abi_builder = abi_builder.free_threaded()?;
        }

        let target_abi = abi_builder.finalize();

        let cygwin = map["cygwin"].as_str() == "True";

        let lib_name = if cfg!(windows) {
            default_lib_name_windows(
                target_abi,
                map["mingw"].as_str() == "True",
                // This is the best heuristic currently available to detect debug build
                // on Windows from sysconfig - e.g. ext_suffix may be
                // `_d.cp312-win_amd64.pyd` for 3.12 debug build
                map["ext_suffix"].starts_with("_d."),
            )?
        } else {
            default_lib_name_unix(
                target_abi,
                cygwin,
                map.get("ld_version").map(String::as_str),
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

        Ok(InterpreterConfigBuilder::new(implementation, version)
            .target_abi(target_abi)?
            .shared(shared)
            .lib_name(Some(lib_name))
            .lib_dir(lib_dir)
            .executable(map.get("executable").cloned())
            .pointer_width(calcsize_pointer * 8)
            .build_flags(BuildFlags::from_interpreter(interpreter)?)?
            .python_framework_prefix(python_framework_prefix)
            .finalize())
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
        let cygwin = soabi.ends_with("cygwin");
        let mut abi_builder = PythonAbiBuilder::from_build_env(implementation, version)?;
        if gil_disabled && abi_builder.kind.is_none() {
            abi_builder = abi_builder.free_threaded()?;
        }
        let target_abi = abi_builder.finalize();
        let lib_name = Some(default_lib_name_unix(
            target_abi,
            cygwin,
            sysconfigdata.get_value("LDVERSION"),
        )?);
        let pointer_width =
            parse_key!(sysconfigdata, "SIZEOF_VOID_P").map(|bytes_width: u32| bytes_width * 8)?;
        let build_flags = BuildFlags::from_sysconfigdata(sysconfigdata);

        Ok(InterpreterConfigBuilder::new(implementation, version)
            .target_abi(target_abi)?
            .shared(shared || framework)
            .lib_dir(lib_dir)
            .lib_name(lib_name)
            .pointer_width(pointer_width)
            .build_flags(build_flags)?
            .python_framework_prefix(python_framework_prefix)
            .finalize())
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
            let mut abi_builder = PythonAbiBuilder::from_build_env(
                config.target_abi.implementation,
                get_abi3_version().unwrap_or(config.target_abi.version),
            )?;
            // only allow free-threaded builds if the build environment didn't force an abi3 build
            if config.target_abi.kind.is_free_threaded() && abi_builder.kind.is_none() {
                abi_builder = abi_builder.free_threaded()?;
            }
            config.target_abi = abi_builder.finalize();

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
        let mut version: Option<PythonVersion> = None;
        let mut shared = None;
        let mut target_abi = None;
        // deprecated in the struct but we still allow it to support old config files
        let mut abi3: Option<bool> = None;
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
                "target_abi" => parse_value!(target_abi, value),
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
        let target_abi = if !(target_abi.is_some() || abi3.is_some() || build_flags.is_some()) {
            PythonAbiBuilder::new(implementation, version).finalize()
        } else if abi3.is_some() && abi3.unwrap() {
            warn!("abi3 configuration file option is deprecated, set target_abi instead");
            PythonAbiBuilder::new(implementation, version)
                .abi3()
                .unwrap()
                .finalize()
        } else if let Some(ref flags) = build_flags {
            if flags.0.contains(&BuildFlag::Py_GIL_DISABLED) {
                PythonAbiBuilder::new(implementation, version)
                    .free_threaded()
                    .unwrap()
                    .finalize()
            } else {
                // we could avoid this branch with if let chains
                ensure!(
                    !(target_abi.is_some() && abi3.is_some()),
                    "Invalid config that sets both target_abi and abi3."
                );
                target_abi.unwrap_or(PythonAbiBuilder::new(implementation, version).finalize())
            }
        } else {
            ensure!(
                !(target_abi.is_some() && abi3.is_some()),
                "Invalid config that sets both target_abi and abi3."
            );
            target_abi.unwrap()
        };

        let build_flags = build_flags.unwrap_or_default();
        let builder = InterpreterConfigBuilder::new(implementation, version)
            .target_abi(target_abi)?
            .shared(shared.unwrap_or(true))
            .lib_name(lib_name)
            .lib_dir(lib_dir)
            .executable(executable)
            .build_flags(build_flags)?
            .suppress_build_script_link_lines(suppress_build_script_link_lines)
            .extra_build_script_lines(extra_build_script_lines)
            .python_framework_prefix(python_framework_prefix);

        let builder = if let Some(pointer_width) = pointer_width {
            builder.pointer_width(pointer_width)
        } else {
            builder
        };

        Ok(builder.finalize())
    }

    /// Helper function to apply a default lib_name if none is set in `PYO3_CONFIG_FILE`.
    ///
    /// This requires knowledge of the final target, so cannot be done when the config file is
    /// inlined into `pyo3-build-config` at build time and instead needs to be done when
    /// resolving the build config for linking.
    #[cfg(any(test, feature = "resolve-config"))]
    pub(crate) fn apply_default_lib_name_to_config_file(&mut self, target: &Triple) {
        if self.lib_name.is_none() {
            self.lib_name = Some(default_lib_name_for_target(self.target_abi, target));
        }
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
        write_line!(target_abi)?;
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
}

#[derive(Debug)]
pub struct InterpreterConfigBuilder {
    implementation: PythonImplementation,
    version: PythonVersion,
    shared: Option<bool>,
    target_abi: Option<PythonAbi>,
    lib_name: Option<String>,
    lib_dir: Option<String>,
    executable: Option<String>,
    pointer_width: Option<u32>,
    build_flags: Option<BuildFlags>,
    suppress_build_script_link_lines: Option<bool>,
    extra_build_script_lines: Option<Vec<String>>,
    python_framework_prefix: Option<String>,
}

impl InterpreterConfigBuilder {
    pub fn new(
        implementation: PythonImplementation,
        version: PythonVersion,
    ) -> InterpreterConfigBuilder {
        InterpreterConfigBuilder {
            implementation,
            version,
            shared: None,
            target_abi: None,
            lib_name: None,
            lib_dir: None,
            executable: None,
            pointer_width: None,
            build_flags: None,
            suppress_build_script_link_lines: None,
            extra_build_script_lines: None,
            python_framework_prefix: None,
        }
    }

    pub fn target_abi(self, target_abi: PythonAbi) -> Result<InterpreterConfigBuilder> {
        ensure!(
            self.target_abi.is_none(),
            "Target ABI already set to {:?}",
            target_abi
        );
        Ok(InterpreterConfigBuilder {
            target_abi: Some(target_abi),
            ..self
        })
    }

    pub fn abi3(self) -> Result<InterpreterConfigBuilder> {
        let implementation = self.implementation;
        let version = self.version;
        self.target_abi(
            PythonAbiBuilder::new(implementation, version)
                .abi3()
                // this can't panic because abi3() is caleld on a builder with no chosen ABI
                .unwrap()
                .finalize(),
        )
    }

    pub fn free_threaded(self) -> Result<InterpreterConfigBuilder> {
        let implementation: PythonImplementation = self.implementation;
        let version: PythonVersion = self.version;
        self.target_abi(
            PythonAbiBuilder::new(implementation, version)
                .free_threaded()
                // this can't panic because abi3() is called on a builder with no chosen ABI
                .unwrap()
                .finalize(),
        )?
        .build_flags(BuildFlags::default())
    }

    pub fn lib_name(self, lib_name: Option<String>) -> InterpreterConfigBuilder {
        InterpreterConfigBuilder { lib_name, ..self }
    }

    pub fn pointer_width(self, pointer_width: u32) -> InterpreterConfigBuilder {
        InterpreterConfigBuilder {
            pointer_width: Some(pointer_width),
            ..self
        }
    }

    pub fn executable(self, executable: Option<String>) -> InterpreterConfigBuilder {
        InterpreterConfigBuilder { executable, ..self }
    }

    pub fn suppress_build_script_link_lines(
        self,
        suppress_build_script_link_lines: Option<bool>,
    ) -> InterpreterConfigBuilder {
        InterpreterConfigBuilder {
            suppress_build_script_link_lines,
            ..self
        }
    }

    pub fn extra_build_script_lines(
        self,
        extra_build_script_lines: Vec<String>,
    ) -> InterpreterConfigBuilder {
        InterpreterConfigBuilder {
            extra_build_script_lines: Some(extra_build_script_lines),
            ..self
        }
    }

    pub fn lib_dir(self, lib_dir: Option<String>) -> InterpreterConfigBuilder {
        InterpreterConfigBuilder { lib_dir, ..self }
    }

    pub fn shared(self, shared: bool) -> InterpreterConfigBuilder {
        InterpreterConfigBuilder {
            shared: Some(shared),
            ..self
        }
    }

    pub fn build_flags(self, build_flags: BuildFlags) -> Result<InterpreterConfigBuilder> {
        ensure!(self.build_flags.is_none(), "Build flags already set!");
        let build_flags = if build_flags.0.contains(&BuildFlag::Py_GIL_DISABLED) {
            ensure!(self.target_abi.is_some(), "Must target a free-threaded ABI if build flags contain Py_GIL_DISABLED but no target_abi is set");
            ensure!(
                self.target_abi.unwrap().kind
                    == PythonAbiKind::VersionSpecific(GilUsed::FreeThreaded),
                "build_flags contains Py_GIL_DISABLED but target ABI is not free-threaded"
            );
            build_flags
        } else if let Some(target_abi) = self.target_abi {
            let mut flags = build_flags.clone();
            if target_abi.kind.is_free_threaded() {
                flags.0.insert(BuildFlag::Py_GIL_DISABLED);
            }
            flags
        } else {
            build_flags
        };
        Ok(InterpreterConfigBuilder {
            build_flags: Some(build_flags),
            ..self
        })
    }

    pub fn python_framework_prefix(
        self,
        python_framework_prefix: Option<String>,
    ) -> InterpreterConfigBuilder {
        InterpreterConfigBuilder {
            python_framework_prefix,
            ..self
        }
    }

    pub fn finalize(self) -> InterpreterConfig {
        #[allow(deprecated)]
        InterpreterConfig {
            implementation: self.implementation,
            version: self.version,
            shared: self.shared.unwrap_or(true),
            target_abi: self
                .target_abi
                .unwrap_or(PythonAbiBuilder::new(self.implementation, self.version).finalize()),
            abi3: false,
            lib_name: self.lib_name,
            lib_dir: self.lib_dir,
            executable: self.executable,
            pointer_width: self.pointer_width,
            build_flags: self.build_flags.unwrap_or_default(),
            suppress_build_script_link_lines: self
                .suppress_build_script_link_lines
                .unwrap_or(false),
            extra_build_script_lines: self.extra_build_script_lines.unwrap_or(vec![]),
            python_framework_prefix: self.python_framework_prefix,
        }
    }
}

#[derive(Debug)]
pub struct PythonAbiBuilder {
    implementation: PythonImplementation,
    version: PythonVersion,
    kind: Option<PythonAbiKind>,
}

impl PythonAbiBuilder {
    pub fn new(implementation: PythonImplementation, version: PythonVersion) -> PythonAbiBuilder {
        PythonAbiBuilder {
            implementation,
            version,
            kind: None,
        }
    }

    pub fn from_build_env(
        implementation: PythonImplementation,
        version: PythonVersion,
    ) -> Result<PythonAbiBuilder> {
        let builder = PythonAbiBuilder {
            implementation,
            version,
            kind: None,
        };
        if is_abi3() {
            builder.abi3()
        } else {
            Ok(builder)
        }
    }

    pub fn abi3(self) -> Result<PythonAbiBuilder> {
        if self.kind.is_some() {
            bail!(
                "ABI kind already set to {:?}, cannot set to abi3",
                self.kind
            )
        }

        // PyPy and GraalPy don't support abi3; don't adjust the version
        if self.implementation.is_pypy() || self.implementation.is_graalpy() {
            return Ok(PythonAbiBuilder {
                implementation: self.implementation,
                version: self.version,
                kind: self.kind,
            });
        }
        let mut build_version = self.version;
        if self.version.minor > STABLE_ABI_MAX_MINOR {
            warn!("Automatically falling back to abi3-py3{STABLE_ABI_MAX_MINOR} because current Python is higher than the maximum supported");
            build_version.minor = STABLE_ABI_MAX_MINOR;
        }

        Ok(PythonAbiBuilder {
            kind: Some(PythonAbiKind::Abi3),
            version: build_version,
            ..self
        })
    }

    pub fn free_threaded(self) -> Result<PythonAbiBuilder> {
        if self.kind.is_some() {
            bail!(
                "Target ABI already set to {:?}, cannot set to free-threaded",
                self.kind
            )
        }
        if self.version < PythonVersion::PY313 {
            let version = self.version;
            bail!(
                "Cannot target free-threaded builds for Python versions before 3.13, tried to build for {version}"
            )
        }
        Ok(PythonAbiBuilder {
            kind: Some(PythonAbiKind::VersionSpecific(GilUsed::FreeThreaded)),
            ..self
        })
    }

    pub fn finalize(self) -> PythonAbi {
        // default to GIL-enabled version-specific ABI
        let kind = self
            .kind
            .unwrap_or(PythonAbiKind::VersionSpecific(GilUsed::GilEnabled));
        PythonAbi {
            implementation: self.implementation,
            kind,
            version: self.version,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PythonAbi {
    /// The Python implementation flavor.
    ///
    /// Serialized to `implementation`.
    pub implementation: PythonImplementation,

    /// The ABI flavor
    ///
    /// Serialized to `kind`
    pub kind: PythonAbiKind,

    /// Python `X.Y` version. e.g. `3.9`.
    ///
    /// Serialized to `version`.
    pub version: PythonVersion,
}

impl Display for PythonAbi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let implementation = self.implementation;
        let kind = self.kind;
        let version = self.version;
        write!(f, "{implementation}-{kind}-{version}")
    }
}

impl FromStr for PythonAbi {
    type Err = crate::errors::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = value.split("-").collect();
        let implementation = parts[0].parse()?;
        let kind = parts[1].parse()?;
        let version: PythonVersion = parts[2].parse()?;
        Ok(PythonAbi {
            implementation,
            kind,
            version,
        })
    }
}

/// The "kind" of stable ABI. Either abi3 or abi3t currently.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PythonAbiKind {
    /// The original stable ABI, supporting Python 3.2 and up
    Abi3,
    /// Version specific ABI, which is different on the free-threaded build
    VersionSpecific(GilUsed),
}

impl Display for PythonAbiKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PythonAbiKind::Abi3 => write!(f, "abi3"),
            PythonAbiKind::VersionSpecific(gil_disabled) => {
                write!(f, "version_specific({gil_disabled})")
            }
        }
    }
}

impl FromStr for PythonAbiKind {
    type Err = crate::errors::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "abi3" => Ok(PythonAbiKind::Abi3),
            "version_specific(free_threaded)" => {
                Ok(PythonAbiKind::VersionSpecific(GilUsed::FreeThreaded))
            }
            "version_specific(gil_enabled)" => {
                Ok(PythonAbiKind::VersionSpecific(GilUsed::GilEnabled))
            }
            _ => Err(format!("Unrecognized ABI name: {value}").into()),
        }
    }
}

impl PythonAbiKind {
    pub fn is_free_threaded(&self) -> bool {
        match self {
            PythonAbiKind::VersionSpecific(gil_disabled) => *gil_disabled == GilUsed::FreeThreaded,
            PythonAbiKind::Abi3 => false,
        }
    }

    pub fn is_abi3(&self) -> bool {
        matches!(self, PythonAbiKind::Abi3)
    }
}

/// Whether the ABI is for the GIL-enabled or free-threaded build.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GilUsed {
    /// The original PyObject layout
    GilEnabled,
    /// The free-threaded PyObject layout
    FreeThreaded,
}

impl Display for GilUsed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GilUsed::GilEnabled => write!(f, "gil_enabled"),
            GilUsed::FreeThreaded => write!(f, "free_threaded"),
        }
    }
}

impl FromStr for GilUsed {
    type Err = crate::errors::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "gil_enabled" => Ok(GilUsed::GilEnabled),
            "free_threaded" => Ok(GilUsed::FreeThreaded),
            _ => Err(format!("Unrecognized ABI name: {value}").into()),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PythonVersion {
    pub major: u8,
    pub minor: u8,
}

impl PythonVersion {
    pub const PY315: Self = PythonVersion {
        major: 3,
        minor: 15,
    };
    pub const PY314: Self = PythonVersion {
        major: 3,
        minor: 14,
    };
    pub const PY313: Self = PythonVersion {
        major: 3,
        minor: 13,
    };
    pub const PY312: Self = PythonVersion {
        major: 3,
        minor: 12,
    };
    pub const PY311: Self = PythonVersion {
        major: 3,
        minor: 11,
    };
    pub const PY310: Self = PythonVersion {
        major: 3,
        minor: 10,
    };
    pub const PY39: Self = PythonVersion { major: 3, minor: 9 };
    pub const PY38: Self = PythonVersion { major: 3, minor: 8 };
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
    RustPython,
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
            PythonImplementation::RustPython => write!(f, "RustPython"),
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
            "RustPython" => Ok(PythonImplementation::RustPython),
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
    let minor_version = (MINIMUM_SUPPORTED_VERSION.minor..=STABLE_ABI_MAX_MINOR)
        .find(|i| cargo_env_var(&format!("CARGO_FEATURE_ABI3_PY3{i}")).is_some());
    minor_version.map(|minor| PythonVersion { major: 3, minor })
}

/// Checks if the `extension-module` feature is enabled for the PyO3 crate.
///
/// This can be triggered either by:
/// - The `extension-module` Cargo feature (deprecated)
/// - Setting the `PYO3_BUILD_EXTENSION_MODULE` environment variable
///
/// Must be called from a PyO3 crate build script.
pub fn is_extension_module() -> bool {
    cargo_env_var("CARGO_FEATURE_EXTENSION_MODULE").is_some()
        || env_var("PYO3_BUILD_EXTENSION_MODULE").is_some()
}

/// Checks if we need to link to `libpython` for the target.
///
/// Must be called from a PyO3 crate build script.
pub fn is_linking_libpython_for_target(target: &Triple) -> bool {
    target.operating_system == OperatingSystem::Windows
        // See https://github.com/PyO3/pyo3/issues/4068#issuecomment-2051159852
        || target.operating_system == OperatingSystem::Aix
        || target.environment == Environment::Android
        || target.environment == Environment::Androideabi
        || target.operating_system == OperatingSystem::Cygwin
        || matches!(target.operating_system, OperatingSystem::IOS(_))
        || !is_extension_module()
}

/// Checks if we need to discover the Python library directory
/// to link the extension module binary.
///
/// Must be called from a PyO3 crate build script.
fn require_libdir_for_target(target: &Triple) -> bool {
    // With raw-dylib, Windows targets never need a lib dir — the compiler generates
    // import entries directly from `#[link(kind = "raw-dylib")]` attributes.
    if target.operating_system == OperatingSystem::Windows {
        return false;
    }

    is_linking_libpython_for_target(target)
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

        compatible |= matches!(target.operating_system, OperatingSystem::IOS(_));

        !compatible
    }

    /// Converts `lib_dir` member field to an UTF-8 string.
    ///
    /// The conversion can not fail because `PYO3_CROSS_LIB_DIR` variable
    /// is ensured contain a valid UTF-8 string.
    #[allow(dead_code)]
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
///   installation. This variable is only needed if PyO3 cannot determine the version to target
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
#[allow(dead_code)]
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
    #[deprecated(since = "0.29.0", note = "no longer supported by PyO3")]
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
#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(Debug, Clone, Default)]
pub struct BuildFlags(pub HashSet<BuildFlag>);

impl BuildFlags {
    const ALL: [BuildFlag; 4] = [
        BuildFlag::Py_DEBUG,
        BuildFlag::Py_REF_DEBUG,
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(unused_mut, dead_code)]
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

    let implementation = cross_compile_config
        .implementation
        .unwrap_or(PythonImplementation::CPython);
    let gil_disabled: bool = cross_compile_config.abiflags.as_deref() == Some("t");
    let mut abi_builder = PythonAbiBuilder::from_build_env(implementation, version)?;
    // The build environment might imply an abi3 build, which can't be free-threaded
    if gil_disabled && abi_builder.kind.is_none() {
        abi_builder = abi_builder.free_threaded()?;
    }
    let target_abi = abi_builder.finalize();

    let lib_name = default_lib_name_for_target(target_abi, &cross_compile_config.target);

    let mut lib_dir = cross_compile_config.lib_dir_string();

    Ok(InterpreterConfigBuilder::new(implementation, version)
        .target_abi(target_abi)?
        .lib_name(Some(lib_name))
        .lib_dir(lib_dir)
        .finalize())
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
    let builder = InterpreterConfigBuilder::new(PythonImplementation::CPython, version)
        .abi3()
        .unwrap();
    // abi3() sets the target_abi on the builder struct so unwrapping is safe
    let target_abi = builder.target_abi.unwrap();
    Ok(if host.operating_system == OperatingSystem::Windows {
        builder.lib_name(Some(default_lib_name_windows(target_abi, false, false)?))
    } else {
        builder
    }
    .finalize())
}

/// Detects the cross compilation target interpreter configuration from all
/// available sources (PyO3 environment variables, Python sysconfigdata, etc.).
///
/// Returns the "default" target interpreter configuration for Windows and
/// when no target Python interpreter is found.
///
/// Must be called from a PyO3 crate build script.
#[allow(dead_code)]
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

    Ok(config)
}

// These contains only the limited ABI symbols.
const WINDOWS_STABLE_ABI_LIB_NAME: &str = "python3";
const WINDOWS_STABLE_ABI_DEBUG_LIB_NAME: &str = "python3_d";

/// Generates the default library name for the target platform.
#[allow(dead_code)]
fn default_lib_name_for_target(abi: PythonAbi, target: &Triple) -> String {
    if target.operating_system == OperatingSystem::Windows {
        default_lib_name_windows(abi, false, false).unwrap()
    } else {
        default_lib_name_unix(
            abi,
            target.operating_system == OperatingSystem::Cygwin,
            None,
        )
        .unwrap()
    }
}

fn default_lib_name_windows(abi: PythonAbi, mingw: bool, debug: bool) -> Result<String> {
    if abi.implementation.is_pypy() {
        // PyPy on Windows ships `libpypy3.X-c.dll` (e.g. `libpypy3.11-c.dll`),
        // not CPython's `pythonXY.dll`. With raw-dylib linking we need the real
        // DLL name rather than the import-library alias.
        Ok(format!(
            "libpypy{}.{}-c",
            abi.version.major, abi.version.minor
        ))
    } else if debug && abi.version < PythonVersion::PY310 {
        // CPython bug: linking against python3_d.dll raises error
        // https://github.com/python/cpython/issues/101614
        Ok(format!(
            "python{}{}_d",
            abi.version.major, abi.version.minor
        ))
    } else if matches!(abi.kind, PythonAbiKind::Abi3) && !abi.implementation.is_graalpy() {
        if debug {
            Ok(WINDOWS_STABLE_ABI_DEBUG_LIB_NAME.to_owned())
        } else {
            Ok(WINDOWS_STABLE_ABI_LIB_NAME.to_owned())
        }
    } else if mingw {
        ensure!(
            !abi.kind.is_free_threaded(),
            "MinGW free-threaded builds are not currently tested or supported"
        );
        // https://packages.msys2.org/base/mingw-w64-python
        Ok(format!("python{}.{}", abi.version.major, abi.version.minor))
    } else if abi.kind.is_free_threaded() {
        ensure!(abi.version >= PythonVersion::PY313, "Cannot compile C extensions for the free-threaded build on Python versions earlier than 3.13, found {}.{}", abi.version.major, abi.version.minor);
        if debug {
            Ok(format!(
                "python{}{}t_d",
                abi.version.major, abi.version.minor
            ))
        } else {
            Ok(format!("python{}{}t", abi.version.major, abi.version.minor))
        }
    } else if debug {
        Ok(format!(
            "python{}{}_d",
            abi.version.major, abi.version.minor
        ))
    } else {
        Ok(format!("python{}{}", abi.version.major, abi.version.minor))
    }
}

fn default_lib_name_unix(abi: PythonAbi, cygwin: bool, ld_version: Option<&str>) -> Result<String> {
    match abi.implementation {
        PythonImplementation::CPython => match ld_version {
            Some(ld_version) => Ok(format!("python{ld_version}")),
            None => {
                if cygwin && matches!(abi.kind, PythonAbiKind::Abi3) {
                    Ok("python3".to_string())
                } else if abi.kind.is_free_threaded() {
                    ensure!(abi.version >= PythonVersion::PY313, "Cannot compile C extensions for the free-threaded build on Python versions earlier than 3.13, found {}.{}", abi.version.major, abi.version.minor);
                    Ok(format!(
                        "python{}.{}t",
                        abi.version.major, abi.version.minor
                    ))
                } else {
                    Ok(format!("python{}.{}", abi.version.major, abi.version.minor))
                }
            }
        },
        PythonImplementation::PyPy => match ld_version {
            Some(ld_version) => Ok(format!("pypy{ld_version}-c")),
            None => Ok(format!("pypy{}.{}-c", abi.version.major, abi.version.minor)),
        },

        PythonImplementation::GraalPy => Ok("python-native".to_string()),
        PythonImplementation::RustPython => Ok("rustpython-capi".to_string()),
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

    let interpreter_config = InterpreterConfig::from_interpreter(interpreter_path, abi3_version)?;

    Ok(interpreter_config)
}

/// Generates an interpreter config suitable for cross-compilation.
///
/// This must be called from PyO3's build script, because it relies on environment variables such as
/// CARGO_CFG_TARGET_OS which aren't available at any other time.
#[allow(dead_code)]
pub fn make_cross_compile_config() -> Result<Option<InterpreterConfig>> {
    let interpreter_config = if let Some(cross_config) = cross_compiling_from_cargo_env()? {
        let mut config = load_cross_compile_config(cross_config)?;
        let mut abi_builder = PythonAbiBuilder::from_build_env(
            config.target_abi.implementation,
            get_abi3_version().unwrap_or(config.target_abi.version),
        )?;
        if config.target_abi.kind.is_free_threaded() && abi_builder.kind.is_none() {
            abi_builder = abi_builder.free_threaded()?;
        }
        config.target_abi = abi_builder.finalize();

        Some(config)
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
    // Unix stable ABI extension modules can usually be built without any interpreter.
    let need_interpreter = abi3_version.is_none() || require_libdir_for_target(&host);

    if have_python_interpreter() {
        match get_host_interpreter(abi3_version) {
            Ok(interpreter_config) => return Ok(interpreter_config),
            // Bail if the interpreter configuration is required to build.
            Err(e) if need_interpreter => return Err(e),
            _ => {
                // Fall back to the stable ABI just as if `PYO3_NO_PYTHON`
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

    let interpreter_config = default_abi3_config(&host, abi3_version.unwrap())?;

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
    assert_eq!(escaped.len() % 2, 0, "invalid hex encoding");

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
        let implementation = PythonImplementation::CPython;
        let version = MINIMUM_SUPPORTED_VERSION;
        let config = InterpreterConfigBuilder::new(implementation, version)
            .abi3()
            .unwrap()
            .pointer_width(32)
            .executable(Some("executable".into()))
            .lib_dir(Some("lib_name".into()))
            .lib_name(Some("lib_name".into()))
            .extra_build_script_lines(vec!["cargo:test1".to_string(), "cargo:test2".to_string()])
            .finalize();
        let mut buf: Vec<u8> = Vec::new();
        config.to_writer(&mut buf).unwrap();

        assert_eq!(config, InterpreterConfig::from_reader(&*buf).unwrap());

        // And some different options, for variety
        let version = PythonVersion::PY310;
        let implementation = PythonImplementation::PyPy;
        let build_flags = {
            let mut flags = HashSet::new();
            flags.insert(BuildFlag::Py_DEBUG);
            flags.insert(BuildFlag::Other(String::from("Py_SOME_FLAG")));
            BuildFlags(flags)
        };
        let config = InterpreterConfigBuilder::new(implementation, version)
            .build_flags(build_flags)
            .unwrap()
            .finalize();

        let mut buf: Vec<u8> = Vec::new();
        config.to_writer(&mut buf).unwrap();

        assert_eq!(config, InterpreterConfig::from_reader(&*buf).unwrap());
    }

    #[test]
    fn test_config_file_roundtrip_with_escaping() {
        let implementation = PythonImplementation::CPython;
        let version = MINIMUM_SUPPORTED_VERSION;
        let config = InterpreterConfigBuilder::new(implementation, version)
            .abi3()
            .unwrap()
            .pointer_width(32)
            .executable(Some("executable".into()))
            .lib_name(Some("lib_name".into()))
            .lib_dir(Some("lib_dir\\n".into()))
            .extra_build_script_lines(vec!["cargo:test1".to_string(), "cargo:test2".to_string()])
            .finalize();
        let mut buf: Vec<u8> = Vec::new();
        config.to_writer(&mut buf).unwrap();

        let buf = unescape(&escape(&buf));

        assert_eq!(config, InterpreterConfig::from_reader(&*buf).unwrap());
    }

    #[test]
    fn test_config_file_defaults() {
        // Only version is required
        let implementation = PythonImplementation::CPython;
        let version = PythonVersion::PY38;
        assert_eq!(
            InterpreterConfig::from_reader("version=3.8".as_bytes()).unwrap(),
            InterpreterConfigBuilder::new(implementation, version,).finalize()
        )
    }

    #[test]
    fn test_config_file_unknown_keys() {
        // ext_suffix is unknown to pyo3-build-config, but it shouldn't error
        let implementation = PythonImplementation::CPython;
        let version = PythonVersion::PY38;
        assert_eq!(
            InterpreterConfig::from_reader("version=3.8\next_suffix=.python38.so".as_bytes())
                .unwrap(),
            InterpreterConfigBuilder::new(implementation, version,).finalize()
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
        sysconfigdata.insert("SOABI", "cpython-38-x86_64-linux-gnu");
        sysconfigdata.insert("VERSION", "3.8");
        sysconfigdata.insert("Py_ENABLE_SHARED", "1");
        sysconfigdata.insert("LIBDIR", "/usr/lib");
        sysconfigdata.insert("LDVERSION", "3.8");
        sysconfigdata.insert("SIZEOF_VOID_P", "8");
        let implementation = PythonImplementation::CPython;
        let version = PythonVersion::PY38;
        assert_eq!(
            InterpreterConfig::from_sysconfigdata(&sysconfigdata).unwrap(),
            InterpreterConfigBuilder::new(implementation, version,)
                .build_flags(BuildFlags::from_sysconfigdata(&sysconfigdata))
                .unwrap()
                .lib_dir(Some("/usr/lib".into()))
                .lib_name(Some("python3.8".into()))
                .pointer_width(64)
                .finalize()
        );
    }

    #[test]
    fn config_from_sysconfigdata_framework() {
        let mut sysconfigdata = Sysconfigdata::new();
        sysconfigdata.insert("SOABI", "cpython-38-x86_64-linux-gnu");
        sysconfigdata.insert("VERSION", "3.8");
        // PYTHONFRAMEWORK should override Py_ENABLE_SHARED
        sysconfigdata.insert("Py_ENABLE_SHARED", "0");
        sysconfigdata.insert("PYTHONFRAMEWORK", "Python");
        sysconfigdata.insert("LIBDIR", "/usr/lib");
        sysconfigdata.insert("LDVERSION", "3.8");
        sysconfigdata.insert("SIZEOF_VOID_P", "8");
        let implementation = PythonImplementation::CPython;
        let version = PythonVersion::PY38;
        assert_eq!(
            InterpreterConfig::from_sysconfigdata(&sysconfigdata).unwrap(),
            InterpreterConfigBuilder::new(implementation, version,)
                .build_flags(BuildFlags::from_sysconfigdata(&sysconfigdata))
                .unwrap()
                .lib_dir(Some("/usr/lib".into()))
                .lib_name(Some("python3.8".into()))
                .pointer_width(64)
                .finalize()
        );

        sysconfigdata = Sysconfigdata::new();
        sysconfigdata.insert("SOABI", "cpython-38-x86_64-linux-gnu");
        sysconfigdata.insert("VERSION", "3.8");
        // An empty PYTHONFRAMEWORK means it is not a framework
        sysconfigdata.insert("Py_ENABLE_SHARED", "0");
        sysconfigdata.insert("PYTHONFRAMEWORK", "");
        sysconfigdata.insert("LIBDIR", "/usr/lib");
        sysconfigdata.insert("LDVERSION", "3.8");
        sysconfigdata.insert("SIZEOF_VOID_P", "8");
        let implementation = PythonImplementation::CPython;
        let version = PythonVersion::PY38;
        assert_eq!(
            InterpreterConfig::from_sysconfigdata(&sysconfigdata).unwrap(),
            InterpreterConfigBuilder::new(implementation, version,)
                .build_flags(BuildFlags::from_sysconfigdata(&sysconfigdata))
                .unwrap()
                .lib_dir(Some("/usr/lib".into()))
                .lib_name(Some("python3.8".into()))
                .pointer_width(64)
                .shared(false)
                .finalize()
        );
    }

    #[test]
    fn windows_hardcoded_abi3_compile() {
        let host = triple!("x86_64-pc-windows-msvc");
        let min_version = "3.8".parse().unwrap();

        let implementation = PythonImplementation::CPython;
        let version = PythonVersion::PY38;
        let config = InterpreterConfigBuilder::new(implementation, version)
            .abi3()
            .unwrap()
            .lib_name(Some("python3".into()))
            .finalize();
        assert_eq!(default_abi3_config(&host, min_version).unwrap(), config);
    }

    #[test]
    fn unix_hardcoded_abi3_compile() {
        let host = triple!("x86_64-unknown-linux-gnu");
        let min_version = "3.9".parse().unwrap();
        let implementation = PythonImplementation::CPython;
        let version = PythonVersion::PY39;
        let config = InterpreterConfigBuilder::new(implementation, version)
            .abi3()
            .unwrap()
            .finalize();
        assert_eq!(default_abi3_config(&host, min_version).unwrap(), config);
    }

    #[test]
    fn windows_hardcoded_cross_compile() {
        let env_vars = CrossCompileEnvVars {
            pyo3_cross: None,
            pyo3_cross_lib_dir: Some("C:\\some\\path".into()),
            pyo3_cross_python_implementation: None,
            pyo3_cross_python_version: Some("3.8".into()),
        };

        let host = triple!("x86_64-unknown-linux-gnu");
        let target = triple!("i686-pc-windows-msvc");
        let cross_config =
            CrossCompileConfig::try_from_env_vars_host_target(env_vars, &host, &target)
                .unwrap()
                .unwrap();

        let implementation = PythonImplementation::CPython;
        let version = PythonVersion::PY38;
        let config = InterpreterConfigBuilder::new(implementation, version)
            .lib_name(Some("python38".into()))
            .lib_dir(Some("C:\\some\\path".into()))
            .finalize();
        assert_eq!(default_cross_compile(&cross_config).unwrap(), config);
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

        let implementation = PythonImplementation::CPython;
        let version = PythonVersion::PY38;
        let config = InterpreterConfigBuilder::new(implementation, version)
            .lib_name(Some("python38".into()))
            .lib_dir(Some("/usr/lib/mingw".into()))
            .finalize();
        assert_eq!(default_cross_compile(&cross_config).unwrap(), config);
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

        let implementation = PythonImplementation::CPython;
        let version = PythonVersion::PY39;
        let config = InterpreterConfigBuilder::new(implementation, version)
            .lib_name(Some("python3.9".into()))
            .lib_dir(Some("/usr/arm64/lib".into()))
            .finalize();
        assert_eq!(default_cross_compile(&cross_config).unwrap(), config);
    }

    #[test]
    fn pypy_hardcoded_cross_compile() {
        let env_vars = CrossCompileEnvVars {
            pyo3_cross: None,
            pyo3_cross_lib_dir: None,
            pyo3_cross_python_implementation: Some("PyPy".into()),
            pyo3_cross_python_version: Some("3.11".into()),
        };

        let triple = triple!("x86_64-unknown-linux-gnu");
        let cross_config =
            CrossCompileConfig::try_from_env_vars_host_target(env_vars, &triple, &triple)
                .unwrap()
                .unwrap();

        let implementation = PythonImplementation::PyPy;
        let version = PythonVersion::PY311;
        let config = InterpreterConfigBuilder::new(implementation, version)
            .lib_name(Some("pypy3.11-c".into()))
            .finalize();
        assert_eq!(default_cross_compile(&cross_config).unwrap(), config);
    }

    #[test]
    fn default_lib_name_windows() {
        assert_eq!(
            super::default_lib_name_windows(
                PythonAbiBuilder::new(PythonImplementation::CPython, PythonVersion::PY39)
                    .finalize(),
                false,
                false,
            )
            .unwrap(),
            "python39",
        );
        // free-threaded Python 3.9 builds should be impossible
        assert!(
            PythonAbiBuilder::new(PythonImplementation::CPython, PythonVersion::PY39)
                .free_threaded()
                .is_err()
        );
        assert_eq!(
            super::default_lib_name_windows(
                PythonAbiBuilder::new(PythonImplementation::CPython, PythonVersion::PY39)
                    .abi3()
                    .unwrap()
                    .finalize(),
                false,
                false,
            )
            .unwrap(),
            "python3",
        );
        assert_eq!(
            super::default_lib_name_windows(
                PythonAbiBuilder::new(PythonImplementation::CPython, PythonVersion::PY39)
                    .finalize(),
                true,
                false,
            )
            .unwrap(),
            "python3.9",
        );
        assert_eq!(
            super::default_lib_name_windows(
                PythonAbiBuilder::new(PythonImplementation::CPython, PythonVersion::PY39)
                    .abi3()
                    .unwrap()
                    .finalize(),
                true,
                false,
            )
            .unwrap(),
            "python3",
        );
        assert_eq!(
            super::default_lib_name_windows(
                PythonAbiBuilder::new(PythonImplementation::PyPy, PythonVersion::PY39)
                    .abi3()
                    .unwrap()
                    .finalize(),
                false,
                false,
            )
            .unwrap(),
            "libpypy3.9-c",
        );
        assert_eq!(
            super::default_lib_name_windows(
                PythonAbiBuilder::new(PythonImplementation::PyPy, PythonVersion::PY311)
                    .abi3()
                    .unwrap()
                    .finalize(),
                false,
                false,
            )
            .unwrap(),
            "libpypy3.11-c",
        );
        assert_eq!(
            super::default_lib_name_windows(
                PythonAbiBuilder::new(PythonImplementation::CPython, PythonVersion::PY39)
                    .abi3()
                    .unwrap()
                    .finalize(),
                false,
                true,
            )
            .unwrap(),
            "python39_d",
        );
        // abi3 debug builds on windows use version-specific lib on 3.9 and older
        // to workaround https://github.com/python/cpython/issues/101614
        assert_eq!(
            super::default_lib_name_windows(
                PythonAbiBuilder::new(PythonImplementation::CPython, PythonVersion::PY39)
                    .abi3()
                    .unwrap()
                    .finalize(),
                false,
                true,
            )
            .unwrap(),
            "python39_d",
        );
        assert_eq!(
            super::default_lib_name_windows(
                PythonAbiBuilder::new(PythonImplementation::CPython, PythonVersion::PY310)
                    .abi3()
                    .unwrap()
                    .finalize(),
                false,
                true,
            )
            .unwrap(),
            "python3_d",
        );
        // mingw and free-threading are incompatible (until someone adds support)
        assert!(super::default_lib_name_windows(
            PythonAbiBuilder::new(PythonImplementation::CPython, PythonVersion::PY313)
                .free_threaded()
                .unwrap()
                .finalize(),
            true,
            false,
        )
        .is_err());
        assert_eq!(
            super::default_lib_name_windows(
                PythonAbiBuilder::new(PythonImplementation::CPython, PythonVersion::PY313)
                    .free_threaded()
                    .unwrap()
                    .finalize(),
                false,
                false,
            )
            .unwrap(),
            "python313t",
        );
        assert_eq!(
            super::default_lib_name_windows(
                PythonAbiBuilder::new(PythonImplementation::CPython, PythonVersion::PY313)
                    .free_threaded()
                    .unwrap()
                    .finalize(),
                false,
                true,
            )
            .unwrap(),
            "python313t_d",
        );
    }

    #[test]
    fn default_lib_name_unix() {
        // Defaults to pythonX.Y for CPython 3.8+
        assert_eq!(
            super::default_lib_name_unix(
                PythonAbiBuilder::new(PythonImplementation::CPython, PythonVersion::PY38)
                    .finalize(),
                false,
                None,
            )
            .unwrap(),
            "python3.8",
        );
        assert_eq!(
            super::default_lib_name_unix(
                PythonAbiBuilder::new(PythonImplementation::CPython, PythonVersion::PY39)
                    .finalize(),
                false,
                None,
            )
            .unwrap(),
            "python3.9",
        );
        // Can use ldversion to override for CPython
        assert_eq!(
            super::default_lib_name_unix(
                PythonAbiBuilder::new(PythonImplementation::CPython, PythonVersion::PY39)
                    .finalize(),
                false,
                Some("3.8d"),
            )
            .unwrap(),
            "python3.8d",
        );

        // PyPy 3.11 includes ldversion
        assert_eq!(
            super::default_lib_name_unix(
                PythonAbiBuilder::new(PythonImplementation::PyPy, PythonVersion::PY311).finalize(),
                false,
                None,
            )
            .unwrap(),
            "pypy3.11-c",
        );

        assert_eq!(
            super::default_lib_name_unix(
                PythonAbiBuilder::new(PythonImplementation::PyPy, PythonVersion::PY39).finalize(),
                false,
                Some("3.11d"),
            )
            .unwrap(),
            "pypy3.11d-c",
        );

        // free-threading adds a t suffix
        assert_eq!(
            super::default_lib_name_unix(
                PythonAbiBuilder::new(PythonImplementation::CPython, PythonVersion::PY313)
                    .free_threaded()
                    .unwrap()
                    .finalize(),
                false,
                None,
            )
            .unwrap(),
            "python3.13t",
        );
        // cygwin abi3 links to unversioned libpython
        assert_eq!(
            super::default_lib_name_unix(
                PythonAbiBuilder::new(PythonImplementation::CPython, PythonVersion::PY313)
                    .abi3()
                    .unwrap()
                    .finalize(),
                true,
                None,
            )
            .unwrap(),
            "python3",
        );
    }

    #[test]
    fn abi_builder_error_paths() {
        let builder = PythonAbiBuilder::new(PythonImplementation::CPython, PythonVersion::PY314)
            .free_threaded()
            .unwrap()
            .abi3();
        assert!(builder.is_err());
        assert!(builder
            .unwrap_err()
            .to_string()
            .contains("ABI already chosen!"));

        let builder = PythonAbiBuilder::new(PythonImplementation::CPython, PythonVersion::PY39)
            .free_threaded();

        assert!(builder.is_err());
        assert!(builder.unwrap_err().to_string().contains("Cannot target"));
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
    fn target_abi3_version_different_from_host() {
        let implementation = PythonImplementation::CPython;
        let host_version = PythonVersion::PY39;
        let target_version = PythonVersion::PY38;
        let config = InterpreterConfigBuilder::new(implementation, host_version)
            .target_abi(
                PythonAbiBuilder::new(implementation, target_version)
                    .abi3()
                    .unwrap()
                    .finalize(),
            )
            .unwrap()
            .finalize();
        assert_eq!(config.target_abi.version, target_version);
        assert_eq!(config.version, host_version);
    }

    #[test]
    fn abi3_version_cannot_be_higher_than_interpreter() {
        if !have_python_interpreter() {
            return;
        }

        let interpreter = get_host_interpreter(Some(PythonVersion {
            major: 3,
            minor: 45,
        }));
        assert!(interpreter.unwrap_err().to_string().contains(
            "cannot set a minimum Python version 3.45 higher than the interpreter version"
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
        let implementation = PythonImplementation::CPython;

        assert_eq!(
            parsed_config,
            InterpreterConfigBuilder::new(
                implementation,
                interpreter_config.version,
                PythonAbiBuilder::new(implementation, interpreter_config.version).finalize()
            )
            .build_flags(interpreter_config.build_flags.0.clone())
            .pointer_width(64)
            .lib_dir(interpreter_config.lib_dir.to_owned())
            .lib_name(interpreter_config.lib_name.to_owned())
            .finalize()
            .unwrap()
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
        let implementation = PythonImplementation::CPython;
        let version = PythonVersion::PY311;
        let interpreter_config = InterpreterConfigBuilder::new(implementation, version)
            .lib_name(Some("python3".into()))
            .finalize();
        assert_eq!(
            interpreter_config.build_script_outputs(),
            [
                "cargo:rustc-cfg=Py_3_8".to_owned(),
                "cargo:rustc-cfg=Py_3_9".to_owned(),
                "cargo:rustc-cfg=Py_3_10".to_owned(),
                "cargo:rustc-cfg=Py_3_11".to_owned(),
            ]
        );

        let interpreter_config = InterpreterConfig {
            target_abi: PythonAbi {
                implementation: PythonImplementation::PyPy,
                ..interpreter_config.target_abi
            },
            ..interpreter_config
        };
        assert_eq!(
            interpreter_config.build_script_outputs(),
            [
                "cargo:rustc-cfg=Py_3_8".to_owned(),
                "cargo:rustc-cfg=Py_3_9".to_owned(),
                "cargo:rustc-cfg=Py_3_10".to_owned(),
                "cargo:rustc-cfg=Py_3_11".to_owned(),
                "cargo:rustc-cfg=PyPy".to_owned(),
            ]
        );
    }

    #[test]
    fn test_build_script_outputs_abi3() {
        let implementation = PythonImplementation::CPython;
        let version = PythonVersion::PY39;
        let interpreter_config = InterpreterConfigBuilder::new(implementation, version)
            .abi3()
            .unwrap()
            .lib_name(Some("python3".into()))
            .finalize();

        assert_eq!(
            interpreter_config.build_script_outputs(),
            [
                "cargo:rustc-cfg=Py_3_8".to_owned(),
                "cargo:rustc-cfg=Py_3_9".to_owned(),
                "cargo:rustc-cfg=Py_LIMITED_API".to_owned(),
            ]
        );

        let interpreter_config = InterpreterConfig {
            target_abi: PythonAbi {
                implementation: PythonImplementation::PyPy,
                ..interpreter_config.target_abi
            },
            ..interpreter_config
        };
        assert_eq!(
            interpreter_config.build_script_outputs(),
            [
                "cargo:rustc-cfg=Py_3_8".to_owned(),
                "cargo:rustc-cfg=Py_3_9".to_owned(),
                "cargo:rustc-cfg=PyPy".to_owned(),
                "cargo:rustc-cfg=Py_LIMITED_API".to_owned(),
            ]
        );
    }

    #[test]
    fn test_build_script_outputs_gil_disabled() {
        let implementation = PythonImplementation::CPython;
        let version = PythonVersion::PY313;
        let interpreter_config = InterpreterConfigBuilder::new(implementation, version)
            .free_threaded()
            .unwrap()
            .lib_name(Some("python3".into()))
            .finalize();
        assert_eq!(
            interpreter_config.build_script_outputs(),
            [
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
    fn test_interpreter_config_builder_gil_disabled_flag() {
        let builder =
            InterpreterConfigBuilder::new(PythonImplementation::CPython, PythonVersion::PY314);
        let mut flags = BuildFlags::new();
        flags.0.insert(BuildFlag::Py_GIL_DISABLED);
        assert!(builder
            .build_flags(flags)
            .unwrap_err()
            .to_string()
            .contains("Must target a free-threaded ABI"));

        let builder =
            InterpreterConfigBuilder::new(PythonImplementation::CPython, PythonVersion::PY314);
        let mut flags = BuildFlags::new();
        flags.0.insert(BuildFlag::Py_GIL_DISABLED);
        assert!(builder
            .abi3()
            .unwrap()
            .build_flags(flags)
            .unwrap_err()
            .to_string()
            .contains("target ABI is not free-threaded"));

        let builder =
            InterpreterConfigBuilder::new(PythonImplementation::CPython, PythonVersion::PY314);
        let config = builder.free_threaded().unwrap();
        assert!(config
            .build_flags
            .unwrap()
            .0
            .contains(&BuildFlag::Py_GIL_DISABLED))
    }

    #[test]
    fn test_build_script_outputs_debug() {
        let mut build_flags = BuildFlags::default();
        build_flags.0.insert(BuildFlag::Py_DEBUG);
        let implementation = PythonImplementation::CPython;
        let version = PythonVersion::PY38;
        let interpreter_config = InterpreterConfigBuilder::new(implementation, version)
            .lib_name(Some("python3".into()))
            .build_flags(build_flags)
            .unwrap()
            .finalize();
        assert_eq!(
            interpreter_config.build_script_outputs(),
            [
                "cargo:rustc-cfg=Py_3_8".to_owned(),
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

    #[test]
    fn test_apply_default_lib_name_to_config_file() {
        let implementation = PythonImplementation::CPython;
        let version = PythonVersion::PY39;
        let mut config = InterpreterConfigBuilder::new(implementation, version).finalize();

        let unix = Triple::from_str("x86_64-unknown-linux-gnu").unwrap();
        let win_x64 = Triple::from_str("x86_64-pc-windows-msvc").unwrap();
        let win_arm64 = Triple::from_str("aarch64-pc-windows-msvc").unwrap();

        config.apply_default_lib_name_to_config_file(&unix);
        assert_eq!(config.lib_name, Some("python3.9".into()));

        config.lib_name = None;
        config.apply_default_lib_name_to_config_file(&win_x64);
        assert_eq!(config.lib_name, Some("python39".into()));

        config.lib_name = None;
        config.apply_default_lib_name_to_config_file(&win_arm64);
        assert_eq!(config.lib_name, Some("python39".into()));

        // PyPy
        config.target_abi.implementation = PythonImplementation::PyPy;
        config.target_abi.version = PythonVersion {
            major: 3,
            minor: 11,
        };
        config.lib_name = None;
        config.apply_default_lib_name_to_config_file(&unix);
        assert_eq!(config.lib_name, Some("pypy3.11-c".into()));

        config.lib_name = None;
        config.apply_default_lib_name_to_config_file(&win_x64);
        assert_eq!(config.lib_name, Some("libpypy3.11-c".into()));

        config.target_abi.implementation = PythonImplementation::CPython;

        // Free-threaded
        config.target_abi.kind = PythonAbiKind::VersionSpecific(GilUsed::FreeThreaded);
        config.target_abi.version = PythonVersion {
            major: 3,
            minor: 13,
        };
        config.lib_name = None;
        config.apply_default_lib_name_to_config_file(&unix);
        assert_eq!(config.lib_name, Some("python3.13t".into()));

        config.lib_name = None;
        config.apply_default_lib_name_to_config_file(&win_x64);
        assert_eq!(config.lib_name, Some("python313t".into()));

        config.lib_name = None;
        config.apply_default_lib_name_to_config_file(&win_arm64);
        assert_eq!(config.lib_name, Some("python313t".into()));

        config.build_flags.0.remove(&BuildFlag::Py_GIL_DISABLED);

        // abi3
        config.target_abi = PythonAbi {
            kind: PythonAbiKind::Abi3,
            ..config.target_abi
        };
        config.lib_name = None;
        config.apply_default_lib_name_to_config_file(&unix);
        assert_eq!(config.lib_name, Some("python3.13".into()));

        config.lib_name = None;
        config.apply_default_lib_name_to_config_file(&win_x64);
        assert_eq!(config.lib_name, Some("python3".into()));

        config.lib_name = None;
        config.apply_default_lib_name_to_config_file(&win_arm64);
        assert_eq!(config.lib_name, Some("python3".into()));
    }
}
