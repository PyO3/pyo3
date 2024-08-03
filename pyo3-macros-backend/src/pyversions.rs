use pyo3_build_config::PythonVersion;

pub fn py_version_ge(major: u8, minor: u8) -> bool {
    pyo3_build_config::get().version >= PythonVersion { major, minor }
}

pub fn is_abi3() -> bool {
    pyo3_build_config::get().abi3
}
