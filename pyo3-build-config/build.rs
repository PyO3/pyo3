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
use impl_::{make_interpreter_config, InterpreterConfig};

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

fn generate_build_configs() -> Result<()> {
    // If PYO3_CONFIG_FILE is set, copy it into the crate.
    let configured = configure(
        InterpreterConfig::from_pyo3_config_file_env().transpose()?,
        "pyo3-build-config-file.txt",
    )?;

    if configured {
        // Don't bother trying to find an interpreter on the host system
        // if the user-provided config file is present.
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
