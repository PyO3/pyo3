use pyo3::init_config::InitConfig;
use pyo3::prelude::Python;

#[test]
#[should_panic(expected = "already initialized")]
fn panics_if_already_init() {
    Python::initialize();
    InitConfig::default().initialize().unwrap();
}
