use std::{
    collections::{HashMap, HashSet},
    convert::AsRef,
    env,
    fs::{self, DirEntry, File},
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    str::FromStr,
};

/// Minimum required Python version.
const PY3_MIN_MINOR: u8 = 6;
/// Maximum Python version that can be used as minimum required Python version with abi3.
const ABI3_MAX_MINOR: u8 = 9;
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

#[derive(Debug, Clone)]
struct PythonVersion {
    major: u8,
    minor: u8,
    implementation: PythonInterpreterKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PythonInterpreterKind {
    CPython,
    PyPy,
}

impl FromStr for PythonInterpreterKind {
    type Err = Box<dyn std::error::Error>;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "CPython" => Ok(PythonInterpreterKind::CPython),
            "PyPy" => Ok(PythonInterpreterKind::PyPy),
            _ => bail!("Invalid interpreter: {}", s),
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
    include_dir: Option<PathBuf>,
    version: Option<String>,
    os: String,
    arch: String,
}

impl CrossCompileConfig {
    fn both() -> Result<Self> {
        Ok(CrossCompileConfig {
            include_dir: env::var_os("PYO3_CROSS_INCLUDE_DIR").map(Into::into),
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
    if target == host {
        // Not cross-compiling
        return Ok(None);
    }

    if target == "i686-pc-windows-msvc" && host == "x86_64-pc-windows-msvc" {
        // Not cross-compiling to compile for 32-bit Python from windows 64-bit
        return Ok(None);
    }

    if host.starts_with(&format!(
        "{}-{}-{}",
        env::var("CARGO_CFG_TARGET_ARCH")?,
        env::var("CARGO_CFG_TARGET_VENDOR")?,
        env::var("CARGO_CFG_TARGET_OS")?
    )) {
        // Not cross-compiling if arch-vendor-os is all the same
        // e.g. x86_64-unknown-linux-musl on x86_64-unknown-linux-gnu host
        return Ok(None);
    }

    if env::var("CARGO_CFG_TARGET_FAMILY")? == "windows" {
        // Windows cross-compile uses both header includes and sysconfig
        return Ok(Some(CrossCompileConfig::both()?));
    }

    // Cross-compiling on any other platform
    Ok(Some(CrossCompileConfig::lib_only()?))
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
struct BuildFlags(HashSet<&'static str>);

impl BuildFlags {
    const ALL: [&'static str; 5] = [
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
    fn from_interpreter(python_path: &Path) -> Result<Self> {
        if env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
            return Ok(Self::windows_hardcoded());
        }

        let mut script = "import sysconfig; \
                        config = sysconfig.get_config_vars();"
            .to_owned();

        for k in BuildFlags::ALL.iter() {
            script.push_str(&format!("print(config.get('{}', '0'));", k));
        }

        let stdout = run_python_script(python_path, &script)?;
        let split_stdout: Vec<&str> = stdout.trim_end().lines().collect();
        if split_stdout.len() != BuildFlags::ALL.len() {
            bail!(
                "Python stdout len didn't return expected number of lines: {}",
                split_stdout.len()
            );
        }
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
        //
        // For the time being, this is the flags as defined in the python source's
        // PC\pyconfig.h. This won't work correctly if someone has built their
        // python with a modified pyconfig.h - sorry if that is you, you will have
        // to comment/uncomment the lines below.
        let mut flags = HashSet::new();
        flags.insert("WITH_THREAD");

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
        Self(flags)
    }

    fn fixup(&mut self, interpreter_config: &InterpreterConfig) {
        if self.0.contains("Py_DEBUG") {
            self.0.insert("Py_REF_DEBUG");
            if interpreter_config.version.major == 3 && interpreter_config.version.minor <= 7 {
                // Py_DEBUG only implies Py_TRACE_REFS until Python 3.7
                self.0.insert("Py_TRACE_REFS");
            }
        }

        // WITH_THREAD is always on for Python 3.7, and for PyPy.
        if (interpreter_config.version.implementation == PythonInterpreterKind::PyPy)
            || (interpreter_config.version.major == 3 && interpreter_config.version.minor >= 7)
        {
            self.0.insert("WITH_THREAD");
        }
    }
}

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
) -> Result<(InterpreterConfig, BuildFlags)> {
    let sysconfig_path = find_sysconfigdata(&cross_compile_config)?;
    let sysconfig_data = parse_sysconfigdata(sysconfig_path)?;

    let major = sysconfig_data.get_numeric("version_major")?;
    let minor = sysconfig_data.get_numeric("version_minor")?;
    let ld_version = match sysconfig_data.get("LDVERSION") {
        Some(s) => s.clone(),
        None => format!("{}.{}", major, minor),
    };
    let calcsize_pointer = sysconfig_data.get_numeric("SIZEOF_VOID_P").ok();

    let python_version = PythonVersion {
        major,
        minor,
        implementation: PythonInterpreterKind::CPython,
    };

    let interpreter_config = InterpreterConfig {
        version: python_version,
        libdir: cross_compile_config.lib_dir.to_str().map(String::from),
        shared: sysconfig_data.get_bool("Py_ENABLE_SHARED")?,
        ld_version,
        base_prefix: "".to_string(),
        executable: PathBuf::new(),
        calcsize_pointer,
    };

    let build_flags = BuildFlags::from_config_map(&sysconfig_data);

    Ok((interpreter_config, build_flags))
}

fn load_cross_compile_from_headers(
    cross_compile_config: CrossCompileConfig,
) -> Result<(InterpreterConfig, BuildFlags)> {
    let python_include_dir = cross_compile_config.include_dir.unwrap();
    let python_include_dir = Path::new(&python_include_dir);
    let patchlevel_defines = parse_header_defines(python_include_dir.join("patchlevel.h"))?;

    let major = patchlevel_defines.get_numeric("PY_MAJOR_VERSION")?;
    let minor = patchlevel_defines.get_numeric("PY_MINOR_VERSION")?;

    let python_version = PythonVersion {
        major,
        minor,
        implementation: PythonInterpreterKind::CPython,
    };

    let config_data = parse_header_defines(python_include_dir.join("pyconfig.h"))?;

    let interpreter_config = InterpreterConfig {
        version: python_version,
        libdir: cross_compile_config.lib_dir.to_str().map(String::from),
        shared: config_data.get_bool("Py_ENABLE_SHARED").unwrap_or(false),
        ld_version: format!("{}.{}", major, minor),
        base_prefix: "".to_string(),
        executable: PathBuf::new(),
        calcsize_pointer: None,
    };

    let build_flags = BuildFlags::from_config_map(&config_data);

    Ok((interpreter_config, build_flags))
}

fn windows_hardcoded_cross_compile(
    cross_compile_config: CrossCompileConfig,
) -> Result<(InterpreterConfig, BuildFlags)> {
    let (major, minor) = if let Some(version) = cross_compile_config.version {
        let mut parts = version.split('.');
        match (
            parts.next().and_then(|major| major.parse().ok()),
            parts.next().and_then(|minor| minor.parse().ok()),
            parts.next(),
        ) {
            (Some(major), Some(minor), None) => (major, minor),
            _ => bail!(
                "Expected major.minor version (e.g. 3.9) for PYO3_CROSS_VERSION, got `{}`",
                version
            ),
        }
    } else if let Some(minor_version) = get_abi3_minor_version() {
        (3, minor_version)
    } else {
        bail!("One of PYO3_CROSS_INCLUDE_DIR, PYO3_CROSS_PYTHON_VERSION, or an abi3-py3* feature must be specified when cross-compiling for Windows.")
    };

    let python_version = PythonVersion {
        major,
        minor,
        implementation: PythonInterpreterKind::CPython,
    };

    let interpreter_config = InterpreterConfig {
        version: python_version,
        libdir: cross_compile_config.lib_dir.to_str().map(String::from),
        shared: true,
        ld_version: format!("{}.{}", major, minor),
        base_prefix: "".to_string(),
        executable: PathBuf::new(),
        calcsize_pointer: None,
    };

    Ok((interpreter_config, BuildFlags::windows_hardcoded()))
}

fn load_cross_compile_info(
    cross_compile_config: CrossCompileConfig,
) -> Result<(InterpreterConfig, BuildFlags)> {
    let target_family = env::var("CARGO_CFG_TARGET_FAMILY")?;
    // Because compiling for windows on linux still includes the unix target family
    if target_family == "unix" {
        // Configure for unix platforms using the sysconfigdata file
        load_cross_compile_from_sysconfigdata(cross_compile_config)
    } else if cross_compile_config.include_dir.is_some() {
        // Must configure by headers on windows platform
        load_cross_compile_from_headers(cross_compile_config)
    } else {
        windows_hardcoded_cross_compile(cross_compile_config)
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
        Ok(ok) if !ok.status.success() => bail!("Python script failed"),
        Ok(ok) => Ok(String::from_utf8(ok.stdout)?),
    }
}

fn get_rustc_link_lib(config: &InterpreterConfig) -> String {
    let link_name = if env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() == "windows" {
        if env::var("CARGO_CFG_TARGET_ENV").unwrap().as_str() == "gnu" {
            // https://packages.msys2.org/base/mingw-w64-python
            format!(
                "pythonXY:python{}.{}",
                config.version.major, config.version.minor
            )
        } else {
            // Link against python3.lib for the stable ABI on Windows.
            // See https://www.python.org/dev/peps/pep-0384/#linkage
            //
            // This contains only the limited ABI symbols.
            if env::var_os("CARGO_FEATURE_ABI3").is_some() {
                "pythonXY:python3".to_owned()
            } else {
                format!(
                    "pythonXY:python{}{}",
                    config.version.major, config.version.minor
                )
            }
        }
    } else {
        match config.version.implementation {
            PythonInterpreterKind::CPython => format!("python{}", config.ld_version),
            PythonInterpreterKind::PyPy => format!("pypy{}-c", config.version.major),
        }
    };

    format!(
        "cargo:rustc-link-lib={link_model}{link_name}",
        link_model = if config.shared { "" } else { "static=" },
        link_name = link_name
    )
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
fn find_interpreter_and_get_config() -> Result<(InterpreterConfig, BuildFlags)> {
    let python_interpreter = find_interpreter()?;
    let interpreter_config = get_config_from_interpreter(&python_interpreter)?;
    if interpreter_config.version.major == 3 {
        return Ok((
            interpreter_config,
            BuildFlags::from_interpreter(&python_interpreter)?,
        ));
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
ANACONDA = os.path.exists(os.path.join(sys.base_prefix, 'conda-meta'))

libdir = sysconfig.get_config_var('LIBDIR')

print("version_major", sys.version_info[0])
print("version_minor", sys.version_info[1])
print("implementation", platform.python_implementation())
if libdir is not None:
    print("libdir", libdir)
print("ld_version", sysconfig.get_config_var('LDVERSION') or sysconfig.get_config_var('py_version_short'))
print("base_prefix", sys.base_prefix)
print("framework", bool(sysconfig.get_config_var('PYTHONFRAMEWORK')))
print("shared", PYPY or ANACONDA or bool(sysconfig.get_config_var('Py_ENABLE_SHARED')))
print("executable", sys.executable)
print("calcsize_pointer", struct.calcsize("P"))
"#;
    let output = run_python_script(interpreter, script)?;
    let map: HashMap<String, String> = parse_script_output(&output);
    let shared = match (
        env::var("CARGO_CFG_TARGET_OS").unwrap().as_str(),
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

    Ok(InterpreterConfig {
        version: PythonVersion {
            major: map["version_major"].parse()?,
            minor: map["version_minor"].parse()?,
            implementation: map["implementation"].parse()?,
        },
        libdir: map.get("libdir").cloned(),
        shared,
        ld_version: map["ld_version"].clone(),
        base_prefix: map["base_prefix"].clone(),
        executable: map["executable"].clone().into(),
        calcsize_pointer: Some(map["calcsize_pointer"].parse()?),
    })
}

fn configure(interpreter_config: &InterpreterConfig) -> Result<()> {
    if interpreter_config.version.major == 2 {
        // fail PYO3_PYTHON=python2 cargo ...
        bail!("Python 2 is not supported");
    }

    if interpreter_config.version.minor < PY3_MIN_MINOR {
        bail!(
            "Python 3 required version is 3.{}, current version is 3.{}",
            PY3_MIN_MINOR,
            interpreter_config.version.minor
        );
    }

    check_target_architecture(interpreter_config)?;
    let target_os = env::var_os("CARGO_CFG_TARGET_OS").unwrap();

    let is_extension_module = env::var_os("CARGO_FEATURE_EXTENSION_MODULE").is_some();
    if !is_extension_module || target_os == "windows" || target_os == "android" {
        println!("{}", get_rustc_link_lib(&interpreter_config));
        if let Some(libdir) = &interpreter_config.libdir {
            println!("cargo:rustc-link-search=native={}", libdir);
        } else if target_os == "windows" {
            println!(
                "cargo:rustc-link-search=native={}\\libs",
                interpreter_config.base_prefix
            );
        }
    }

    if interpreter_config.shared {
        println!("cargo:rustc-cfg=Py_SHARED");
    }

    if interpreter_config.version.implementation == PythonInterpreterKind::PyPy {
        println!("cargo:rustc-cfg=PyPy");
    };

    let minor = if env::var_os("CARGO_FEATURE_ABI3").is_some() {
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

    for i in PY3_MIN_MINOR..=minor {
        println!("cargo:rustc-cfg=Py_3_{}", i);
    }

    Ok(())
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

fn get_abi3_minor_version() -> Option<u8> {
    (PY3_MIN_MINOR..=ABI3_MAX_MINOR)
        .find(|i| env::var_os(format!("CARGO_FEATURE_ABI3_PY3{}", i)).is_some())
}

fn abi3_without_interpreter() -> Result<()> {
    println!("cargo:rustc-cfg=Py_LIMITED_API");
    let mut flags = "FLAG_WITH_THREAD=1".to_string();
    let abi_version = get_abi3_minor_version().unwrap_or(ABI3_MAX_MINOR);
    for minor in PY3_MIN_MINOR..=abi_version {
        println!("cargo:rustc-cfg=Py_3_{}", minor);
        flags += &format!(",CFG_Py_3_{}", minor);
    }
    println!("cargo:rustc-cfg=py_sys_config=\"WITH_THREAD\"");
    println!("cargo:python_flags={}", flags);

    // Unfortunately, on windows we can't build without at least providing
    // python.lib to the linker. While maturin tells the linker the location
    // of python.lib, we need to do the renaming here, otherwise cargo
    // complains that the crate using pyo3 does not contains a `#[link(...)]`
    // attribute with pythonXY.
    if env::var("CARGO_CFG_TARGET_FAMILY")? == "windows" {
        println!("cargo:rustc-link-lib=pythonXY:python3");
    }

    Ok(())
}

fn main() -> Result<()> {
    // If PYO3_NO_PYTHON is set with abi3, we can build PyO3 without calling Python.
    // We only check for the abi3-py3{ABI3_MAX_MINOR} because lower versions depend on it.
    if env::var_os("PYO3_NO_PYTHON").is_some()
        && env::var_os(format!("CARGO_FEATURE_ABI3_PY3{}", ABI3_MAX_MINOR)).is_some()
    {
        return abi3_without_interpreter();
    }
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
    let (interpreter_config, mut build_flags) = if let Some(paths) = cross_compiling()? {
        load_cross_compile_info(paths)?
    } else {
        find_interpreter_and_get_config()?
    };

    build_flags.fixup(&interpreter_config);
    configure(&interpreter_config)?;

    for flag in &build_flags.0 {
        println!("cargo:rustc-cfg={}=\"{}\"", CFG_KEY, flag)
    }

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

    // TODO: this is a hack to workaround compile_error! warnings about auto-initialize on PyPy
    // Once cargo's `resolver = "2"` is stable (~ MSRV Rust 1.52), remove this.
    if env::var_os("PYO3_CI").is_some() {
        println!("cargo:rustc-cfg=__pyo3_ci");
    }

    Ok(())
}
