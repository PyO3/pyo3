use pyo3_build_config::PythonVersion;

pub fn is_abi3_before(major: u8, minor: u8) -> bool {
    let config = pyo3_build_config::get();
    config.abi3 && !config.is_free_threaded() && config.version < PythonVersion { major, minor }
}

pub fn is_py_before(major: u8, minor: u8) -> bool {
    let config = pyo3_build_config::get();
    config.version < PythonVersion { major, minor }
}
