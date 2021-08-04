// Import some modules from this crate inline to generate the build config.
// Allow dead code because not all code in the modules is used in this build script.

#[path = "src/impl_.rs"]
#[allow(dead_code)]
mod impl_;

#[path = "src/errors.rs"]
#[allow(dead_code)]
mod errors;

use std::{env, path::Path};

use errors::{Result, Context};

fn generate_build_config() -> Result<()> {
    // Create new interpreter config and write it to the default location
    let interpreter_config = impl_::make_interpreter_config()?;

    let path = Path::new(&env::var_os("OUT_DIR").unwrap()).join("pyo3-build-config.txt");
    interpreter_config
        .to_writer(&mut std::fs::File::create(&path).with_context(|| {
            format!("failed to create config file at {}", path.display())
        })?)
}

fn main() {
    if let Err(e) = generate_build_config() {
        eprintln!("error: {}", e.report());
        std::process::exit(1)
    }
}
