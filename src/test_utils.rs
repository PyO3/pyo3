// Brings in `test_utils` from the `tests` directory
//
// to make that file function (lots of references to `pyo3` within it) need
// re-bind `crate` as pyo3
use crate as pyo3;
include!("../tests/test_utils/mod.rs");
