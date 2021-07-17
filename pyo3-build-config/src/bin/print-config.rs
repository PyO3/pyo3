use pyo3_build_config::{find_interpreter, get_config_from_interpreter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = get_config_from_interpreter(&find_interpreter()?)?;

    println!("implementation: {}", config.implementation);
    println!("interpreter version: {}", config.version);
    println!("interpreter path: {:?}", config.executable);
    println!("libdir: {:?}", config.libdir);
    println!("shared: {}", config.shared);
    println!("base prefix: {:?}", config.base_prefix);
    println!("ld_version: {:?}", config.ld_version);
    println!("pointer width: {:?}", config.calcsize_pointer);

    Ok(())
}
