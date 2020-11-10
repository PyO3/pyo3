use pyo3::prelude::*;
use pyo3::wrap_pymodule;

pub mod buf_and_str;
pub mod datetime;
pub mod dict_iter;
pub mod misc;
pub mod objstore;
pub mod othermod;
pub mod pyclass_iter;
pub mod subclassing;

use buf_and_str::*;
use datetime::*;
use dict_iter::*;
use misc::*;
use objstore::*;
use othermod::*;
use pyclass_iter::*;
use subclassing::*;

#[pymodule]
fn maturin_extension(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(buf_and_str))?;
    m.add_wrapped(wrap_pymodule!(datetime))?;
    m.add_wrapped(wrap_pymodule!(dict_iter))?;
    m.add_wrapped(wrap_pymodule!(misc))?;
    m.add_wrapped(wrap_pymodule!(objstore))?;
    m.add_wrapped(wrap_pymodule!(othermod))?;
    m.add_wrapped(wrap_pymodule!(pyclass_iter))?;
    m.add_wrapped(wrap_pymodule!(subclassing))?;

    Ok(())
}
