extern crate regex;
use self::regex::Regex;

use std::env;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

use python_version::{PythonInterpreterKind, PythonVersion};
use std::collections::HashMap;
use std::string::String;
use utils::{canonicalize_executable, parse_header_defines, run_python_script};

const PY3_MIN_MINOR: u8 = 5;

const CFG_KEY: &'static str = "py_sys_config";

static SYSCONFIG_VALUES: [&'static str; 1] = [
    // cfg doesn't support flags with values, just bools - so flags
    // below are translated into bools as {varname}_{val}
    //
    // for example, Py_UNICODE_SIZE_2 or Py_UNICODE_SIZE_4
    "Py_UNICODE_SIZE", // note - not present on python 3.3+, which is always wide
];

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

#[derive(Debug, PartialEq)]
pub struct InterpreterConfig {
    pub version: PythonVersion,
    pub path: PathBuf,
    pub libpath: String,
    pub enable_shared: bool,
    pub ld_version: String,
    pub exec_prefix: String,
    pub abi_version: String,
}

impl InterpreterConfig {
    /// Tries to read interpreter configuration from path to interpreter.
    pub fn from_path(interpreter: impl AsRef<Path>) -> Result<InterpreterConfig, String> {
        let version = PythonVersion::from_interpreter(&interpreter)?;
        InterpreterConfig::ensure_python_version_is_supported(&version)?;

        match version.kind {
            PythonInterpreterKind::PyPy => {
                let script = "\
import sysconfig
import sys
import os

def get_pypy_link_lib():
    data_dir = os.path.join(sysconfig.get_path('data'))
    for r, dirs, files in os.walk(data_dir):
        for f in files:
            if 'libpypy-c' in f or 'libpypy3-c' in f:
                return os.path.dirname(os.path.join(r, f))
    raise Exception('cannot locate libpypy')

print(get_pypy_link_lib())
print(sys.exec_prefix)
";
                let out = run_python_script(&interpreter, script)?;
                let lines: Vec<&str> = out.lines().collect();

                let abi_tag = run_python_script(&interpreter, GET_ABI_TAG)?
                    .trim_end()
                    .to_string();

                Ok(InterpreterConfig {
                    version: version.clone(),
                    path: interpreter.as_ref().to_owned(),
                    libpath: lines[0].to_string(),
                    enable_shared: true,
                    ld_version: format!(
                        "{}.{}",
                        &version.major,
                        &version.minor.expect(
                            "Interpreter config was loaded from path, above, so this will be set"
                        )
                    )
                    .to_string(),
                    exec_prefix: lines[1].to_string(),
                    abi_version: abi_tag,
                })
            }
            PythonInterpreterKind::CPython => {
                let script = "import sys; import sysconfig;\
         print(sysconfig.get_config_var('LIBDIR')); \
         print(sysconfig.get_config_var('Py_ENABLE_SHARED')); \
         print(sysconfig.get_config_var('LDVERSION') or sysconfig.get_config_var('py_version_short')); \
         print(sys.exec_prefix);";

                let out = run_python_script(&interpreter, script)?;
                let lines: Vec<&str> = out.lines().collect();

                let abi_tag = run_python_script(&interpreter, GET_ABI_TAG)?
                    .trim_end()
                    .to_string();

                Ok(InterpreterConfig {
                    version: version.clone(),
                    path: interpreter.as_ref().to_owned(),
                    libpath: lines[0].to_string(),
                    enable_shared: lines[1] == "1",
                    ld_version: lines[2].to_string(),
                    abi_version: abi_tag,
                    exec_prefix: lines[3].to_string(),
                })
            }
        }
    }

    pub fn from_cross_compile_info() -> Result<InterpreterConfig, String> {
        let python_include_dir = env::var("PYO3_CROSS_INCLUDE_DIR")
            .map_err(|e| "Need to define `PYO3_CROSS_INCLUDE_DIR`")?;

        let python_include_dir = Path::new(&python_include_dir);
        let patchlevel_defines = parse_header_defines(python_include_dir.join("patchlevel.h"))?;

        let version = PythonVersion::from_cross_env(&patchlevel_defines)?;
        InterpreterConfig::ensure_python_version_is_supported(&version)?;

        let config_map = parse_header_defines(python_include_dir.join("pyconfig.h"))?;

        let enable_shared: bool = config_map
            .get("Py_ENABLE_SHARED")
            .ok_or_else(|| "Py_ENABLE_SHARED undefined".to_string())?
            .parse()
            .map_err(|e| "Failed to `Py_ENABLE_SHARED`".to_string())?;

        let libpath = env::var("PYO3_CROSS_LIB_DIR")
            .map_err(|e| "PYO3_CROSS_LIB_DIR undefined".to_string())?;

        Ok(Self {
            version: version.clone(),
            // compatibility, not used when cross compiling.
            path: PathBuf::new(),
            libpath,
            enable_shared,
            ld_version: format!(
                "{}.{}",
                &version.major,
                &version
                    .minor
                    .expect("Interpreter config was loaded from path, above, so this will be set")
            ),
            // compatibility, not used when cross compiling
            exec_prefix: "".to_string(),
            // compatibility, not used when cross compiling
            abi_version: "".to_string(),
        })
    }

    /// Checks if interpreter is supported by PyO3
    fn ensure_python_version_is_supported(version: &PythonVersion) -> Result<(), String> {
        match (version.kind, version.major, version.minor) {
            (PythonInterpreterKind::PyPy, 2, _) => {
                Err("PyPy cpyext bindings is only supported for Python3".to_string())
            }
            (_, 3, Some(minor)) if minor < PY3_MIN_MINOR => Err(format!(
                "Python 3 required version is 3.{}, current version is 3.{}",
                PY3_MIN_MINOR, minor
            )),
            _ => Ok(()),
        }
    }

    fn is_pypy(&self) -> bool {
        match self.version.kind {
            PythonInterpreterKind::PyPy => true,
            _ => false,
        }
    }

    fn is_py3(&self) -> bool {
        self.version.major == 3
    }

    /// Examine python's compile flags to pass to cfg by launching
    /// the interpreter and printing variables of interest from
    /// sysconfig.get_config_vars.
    #[cfg(not(target_os = "windows"))]
    pub fn get_config_vars(&self) -> Result<HashMap<String, String>, String> {
        let mut script = "import sysconfig; config = sysconfig.get_config_vars();".to_owned();

        for k in SYSCONFIG_FLAGS.iter().chain(SYSCONFIG_VALUES.iter()) {
            script.push_str(&format!(
                "print(config.get('{}', {}))",
                k,
                if is_value(k) { "None" } else { "0" }
            ));
            script.push_str(";");
        }

        let out = try!(run_python_script(&self.path, &script));

        let split_stdout: Vec<&str> = out.trim_end().lines().collect();
        if split_stdout.len() != SYSCONFIG_VALUES.len() + SYSCONFIG_FLAGS.len() {
            return Err(format!(
                "python stdout len didn't return expected number of lines: {}",
                split_stdout.len()
            )
            .to_string());
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

        if self.is_pypy() {
            all_vars.insert("WITH_THREAD".to_owned(), "1".to_owned());
            all_vars.insert("Py_USING_UNICODE".to_owned(), "1".to_owned());
            all_vars.insert("Py_UNICODE_SIZE".to_owned(), "4".to_owned());
            all_vars.insert("Py_UNICODE_WIDE".to_owned(), "1".to_owned());
        };

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
    pub fn get_config_vars(&self) -> Result<HashMap<String, String>, String> {
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

    fn get_pypy_link_library_flag(&self) -> String {
        let library_name = if self.is_py3() { "pypy3-c" } else { "pypy-c" };

        // All modern PyPy versions with cpyext are compiled as shared libraries.
        format!("cargo:rustc-link-lib={}", library_name)
    }

    #[cfg(not(target_os = "macos"))]
    #[cfg(not(target_os = "windows"))]
    fn get_rustc_link_lib(&self) -> Result<String, String> {
        if self.is_pypy() {
            return Ok(self.get_pypy_link_library_flag());
        }

        if self.enable_shared {
            Ok(format!(
                "cargo:rustc-link-lib=python{}",
                interpreter_config.ld_version
            ))
        } else {
            Ok(format!(
                "cargo:rustc-link-lib=static=python{}",
                self.ld_version
            ))
        }
    }

    #[cfg(target_os = "windows")]
    fn get_rustc_link_lib(&self) -> Result<String, String> {
        if self.is_pypy() {
            return Ok(self.get_pypy_link_library_flag());
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
    fn get_macos_linkmodel(&self) -> Result<String, String> {
        let script = "import sysconfig; print('framework' if sysconfig.get_config_var('PYTHONFRAMEWORK') else ('shared' if sysconfig.get_config_var('Py_ENABLE_SHARED') else 'static'));";
        let out = run_python_script(&self.path, script).unwrap();
        Ok(out.trim_end().to_owned())
    }

    #[cfg(target_os = "macos")]
    fn get_rustc_link_lib(&self) -> Result<String, String> {
        if self.is_pypy() {
            return Ok(self.get_pypy_link_library_flag());
        }

        // os x can be linked to a framework or static or dynamic, and
        // Py_ENABLE_SHARED is wrong; framework means shared library
        match self.get_macos_linkmodel().unwrap().as_ref() {
            "static" => Ok(format!(
                "cargo:rustc-link-lib=static=python{}",
                self.ld_version
            )),
            "shared" => Ok(format!("cargo:rustc-link-lib=python{}", self.ld_version)),
            "framework" => Ok(format!("cargo:rustc-link-lib=python{}", self.ld_version)),
            other => Err(format!("unknown linkmodel {}", other)),
        }
    }

    /// print cargo vars to stdout.
    pub fn emit_cargo_vars(&self) -> String {
        let is_extension_module = env::var_os("CARGO_FEATURE_EXTENSION_MODULE").is_some();

        println!("cargo:rustc-cfg={}", self.abi_version);

        if !is_extension_module || cfg!(target_os = "windows") {
            println!("{}", self.get_rustc_link_lib().unwrap());

            if self.libpath != "None" {
                println!("cargo:rustc-link-search=native={}", self.libpath);
            } else if cfg!(target_os = "windows") {
                println!("cargo:rustc-link-search=native={}\\libs", self.exec_prefix);
            }
        }

        let mut flags = String::new();

        if self.is_pypy() {
            println!("cargo:rustc-cfg=PyPy");
            flags += format!("CFG_PyPy").as_ref();
        };

        if self.is_py3() {
            if env::var_os("CARGO_FEATURE_PEP_384").is_some() {
                println!("cargo:rustc-cfg=Py_LIMITED_API");
            }
            println!("cargo:rustc-cfg=Py_3");
            flags += "CFG_Py_3,";

            if let Some(minor) = self.version.minor {
                for i in 5..(minor + 1) {
                    println!("cargo:rustc-cfg=Py_3_{}", i);
                    flags += format!("CFG_Py_3_{},", i).as_ref();
                }
            }
        } else {
            println!("cargo:rustc-cfg=Py_2");
            flags += format!("CFG_Py_2,").as_ref();
        }

        return flags;
    }
}

pub fn is_value(key: &str) -> bool {
    SYSCONFIG_VALUES.iter().any(|x| *x == key)
}

pub fn cfg_line_for_var(key: &str, val: &str) -> Option<String> {
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
pub fn find_interpreter(expected_version: &PythonVersion) -> Result<InterpreterConfig, String> {
    if let Some(interpreter_from_env) = env::var_os("PYTHON_SYS_EXECUTABLE") {
        let interpreter_path_or_executable = interpreter_from_env
            .to_str()
            .expect("PYTHON_SYS_EXECUTABLE has non UTF-8 value");

        let interpreter_path =
            canonicalize_executable(interpreter_path_or_executable).expect(&format!(
                "Could not find interpreter passed in PYTHON_SYS_EXECUTABLE={}\n",
                interpreter_path_or_executable
            ));

        let interpreter_config = InterpreterConfig::from_path(interpreter_path)?;

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

    let possible_python_paths = expected_version.possible_binary_names();

    dbg!(&possible_python_paths);

    for possible_path in possible_python_paths {
        let interpreter_path = canonicalize_executable(&possible_path);
        dbg!(&interpreter_path);
        if let Some(path) = interpreter_path {
            let interpreter_config = InterpreterConfig::from_path(path)?;

            if expected_version == &interpreter_config.version {
                return Ok(interpreter_config);
            }
        }
    }

    Err(format!("No python interpreter found"))
}

// Code copied from python wheel package
const GET_ABI_TAG: &'static str = "
from sysconfig import get_config_var
import platform
import sys

def get_impl_ver():
    impl_ver = get_config_var('py_version_nodot')
    if not impl_ver or get_abbr_impl() == 'pp':
        impl_ver = ''.join(map(str, get_impl_version_info()))
    return impl_ver

def get_flag(var, fallback, expected=True, warn=True):
    val = get_config_var(var)
    if val is None:
        if warn:
            warnings.warn('Config variable {0} is unset, Python ABI tag may '
                          'be incorrect'.format(var), RuntimeWarning, 2)
        return fallback()
    return val == expected


def get_abbr_impl():
    impl = platform.python_implementation()
    if impl == 'PyPy':
        return 'pp'
    elif impl == 'Jython':
        return 'jy'
    elif impl == 'IronPython':
        return 'ip'
    elif impl == 'CPython':
        return 'cp'

    raise LookupError('Unknown Python implementation: ' + impl)

def get_abi_tag():
    soabi = get_config_var('SOABI')
    impl = get_abbr_impl()
    if not soabi and impl in ('cp', 'pp') and hasattr(sys, 'maxunicode'):
        d = ''
        m = ''
        u = ''
        if get_flag('Py_DEBUG',
                    lambda: hasattr(sys, 'gettotalrefcount'),
                    warn=(impl == 'cp')):
            d = 'd'
        if get_flag('WITH_PYMALLOC',
                    lambda: impl == 'cp',
                    warn=(impl == 'cp')):
            m = 'm'
        if get_flag('Py_UNICODE_SIZE',
                    lambda: sys.maxunicode == 0x10ffff,
                    expected=4,
                    warn=(impl == 'cp' and
                          sys.version_info < (3, 3))) \
                and sys.version_info < (3, 3):
            u = 'u'
        abi = '%s%s%s%s%s' % (impl, get_impl_ver(), d, m, u)
    elif soabi and soabi.startswith('cpython-'):
        abi = 'cp' + soabi.split('-')[1]
    elif soabi:
        abi = soabi.replace('.', '_').replace('-', '_')
    else:
        abi = None
    return abi

print(get_abi_tag())
";
