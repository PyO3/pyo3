//! Optional `python3.dll` import library generator for Windows

use std::env;
use std::path::PathBuf;

use python3_dll_a::ImportLibraryGenerator;
use target_lexicon::{Architecture, OperatingSystem, Triple};

use super::{PythonImplementation, PythonVersion};
use crate::errors::{Context, Error, Result};

/// Generates the `python3.dll` or `pythonXY.dll` import library for Windows targets.
///
/// Places the generated import library into the build script output directory
/// and returns the full library directory path.
///
/// Does nothing if the target OS is not Windows.
pub(super) fn generate_import_lib(
    target: &Triple,
    py_impl: PythonImplementation,
    py_version: Option<PythonVersion>,
    abiflags: Option<&str>,
) -> Result<Option<String>> {
    if target.operating_system != OperatingSystem::Windows {
        return Ok(None);
    }

    let out_dir =
        env::var_os("OUT_DIR").expect("generate_import_lib() must be called from a build script");

    // Put the newly created import library into the build script output directory.
    let mut out_lib_dir = PathBuf::from(out_dir);
    out_lib_dir.push("lib");

    // Convert `Architecture` enum to rustc `target_arch` option format.
    let arch = match target.architecture {
        // i686, i586, etc.
        Architecture::X86_32(_) => "x86".to_string(),
        other => other.to_string(),
    };

    let env = target.environment.to_string();
    let implementation = match py_impl {
        PythonImplementation::CPython => python3_dll_a::PythonImplementation::CPython,
        PythonImplementation::PyPy => python3_dll_a::PythonImplementation::PyPy,
        PythonImplementation::GraalPy => {
            return Err(Error::from("No support for GraalPy on Windows"))
        }
    };

    ImportLibraryGenerator::new(&arch, &env)
        .version(py_version.map(|v| (v.major, v.minor)))
        .implementation(implementation)
        .abiflags(abiflags)
        .generate(&out_lib_dir)
        .context("failed to generate python3.dll import library")?;

    let out_lib_dir_string = out_lib_dir
        .to_str()
        .ok_or("build directory is not a valid UTF-8 string")?
        .to_owned();

    Ok(Some(out_lib_dir_string))
}
