use pyo3_build_config::pyo3_build_script_impl::{errors::Result, resolve_interpreter_config};

fn configure_pyo3() -> Result<()> {
    let interpreter_config = resolve_interpreter_config()?;
    interpreter_config.emit_pyo3_cfgs();
    Ok(())
}

fn main() {
    if let Err(e) = configure_pyo3() {
        eprintln!("error: {}", e.report());
        std::process::exit(1)
    }
}
