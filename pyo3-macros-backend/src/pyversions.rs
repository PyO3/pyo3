use pyo3_build_config::PythonVersion;

pub fn is_abi3_before(major: u8, minor: u8) -> bool {
    let config = pyo3_build_config::get();
    config.abi3 && config.version < PythonVersion { major, minor }
}
