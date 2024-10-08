#![no_implicit_prelude]
#![allow(dead_code, unused_variables, clippy::unnecessary_wraps)]

// The modules in this test are used to check PyO3 macro expansion is hygienic. By locating the test
// inside the crate the global `::pyo3` namespace is not available, so in combination with
// #[pyo3(crate = "crate")] this validates that all macro expansion respects the setting.

mod misc;
mod pyclass;
mod pyfunction;
mod pymethods;
mod pymodule;
