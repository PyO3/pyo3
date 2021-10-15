// Import some modules from this crate inline to generate the build config.
// Allow dead code because not all code in the modules is used in this build script.

#[path = "src/impl_.rs"]
#[allow(dead_code)]
mod impl_;

#[path = "src/errors.rs"]
#[allow(dead_code)]
mod errors;

use std::{env, path::Path};

use errors::{Context, Result};
use impl_::{
    env_var, get_abi3_version, make_interpreter_config, BuildFlags, InterpreterConfig,
    PythonImplementation,
};

fn configure(interpreter_config: Option<InterpreterConfig>, name: &str) -> Result<bool> {
    let target = Path::new(&env::var_os("OUT_DIR").unwrap()).join(name);
    if let Some(config) = interpreter_config {
        config
            .to_writer(&mut std::fs::File::create(&target).with_context(|| {
                format!("failed to write config file at {}", target.display())
            })?)?;
        Ok(true)
    } else {
        std::fs::File::create(&target)
            .with_context(|| format!("failed to create new file at {}", target.display()))?;
        Ok(false)
    }
}

/// If PYO3_CONFIG_FILE is set, copy it into the crate.
fn config_file() -> Result<Option<InterpreterConfig>> {
    if let Some(path) = env_var("PYO3_CONFIG_FILE") {
        let path = Path::new(&path);
        println!("cargo:rerun-if-changed={}", path.display());
        // Absolute path is necessary because this build script is run with a cwd different to the
        // original `cargo build` instruction.
        ensure!(
            path.is_absolute(),
            "PYO3_CONFIG_FILE must be an absolute path"
        );

        let interpreter_config = InterpreterConfig::from_path(path)
            .context("failed to parse contents of PYO3_CONFIG_FILE")?;
        Ok(Some(interpreter_config))
    } else {
        Ok(None)
    }
}

/// If PYO3_NO_PYTHON is set with abi3, use standard abi3 settings.
pub fn abi3_config() -> Option<InterpreterConfig> {
    if let Some(version) = get_abi3_version() {
        if env_var("PYO3_NO_PYTHON").is_some() {
            return Some(InterpreterConfig {
                version,
                // NB PyPy doesn't support abi3 yet
                implementation: PythonImplementation::CPython,
                abi3: true,
                lib_name: None,
                lib_dir: None,
                build_flags: BuildFlags::abi3(),
                pointer_width: None,
                executable: None,
                shared: true,
                suppress_build_script_link_lines: false,
                extra_build_script_lines: vec![],
            });
        }
    }
    None
}

fn generate_build_configs() -> Result<()> {
    let mut configured = false;
    configured |= configure(config_file()?, "pyo3-build-config-file.txt")?;
    configured |= configure(abi3_config(), "pyo3-build-config-abi3.txt")?;

    if configured {
        // Don't bother trying to find an interpreter on the host system if at least one of the
        // config file or abi3 settings are present
        configure(None, "pyo3-build-config.txt")?;
    } else {
        configure(Some(make_interpreter_config()?), "pyo3-build-config.txt")?;
    }
    Ok(())
}

fn main() {
    if std::env::var("CARGO_FEATURE_RESOLVE_CONFIG").is_ok() {
        if let Err(e) = generate_build_configs() {
            eprintln!("error: {}", e.report());
            std::process::exit(1)
        }
    } else {
        eprintln!("resolve-config feature not enabled; build script in no-op mode");
    }
}
