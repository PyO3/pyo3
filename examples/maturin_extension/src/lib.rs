use pyo3::prelude::*;
use pyo3::types::PyDict;
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
fn maturin_extension(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(buf_and_str))?;
    m.add_wrapped(wrap_pymodule!(datetime))?;
    m.add_wrapped(wrap_pymodule!(dict_iter))?;
    m.add_wrapped(wrap_pymodule!(misc))?;
    m.add_wrapped(wrap_pymodule!(objstore))?;
    m.add_wrapped(wrap_pymodule!(othermod))?;
    m.add_wrapped(wrap_pymodule!(pyclass_iter))?;
    m.add_wrapped(wrap_pymodule!(subclassing))?;

    // Inserting to sys.modules allows importing submodules nicely from Python
    // e.g. import maturin_extension.buf_and_str as bas

    let sys = PyModule::import(py, "sys")?;
    let sys_modules: &PyDict = sys.getattr("modules")?.downcast()?;
    sys_modules.set_item("maturin_extension.buf_and_str", m.getattr("buf_and_str")?)?;
    sys_modules.set_item("maturin_extension.datetime", m.getattr("datetime")?)?;
    sys_modules.set_item("maturin_extension.dict_iter", m.getattr("dict_iter")?)?;
    sys_modules.set_item("maturin_extension.misc", m.getattr("misc")?)?;
    sys_modules.set_item("maturin_extension.objstore", m.getattr("objstore")?)?;
    sys_modules.set_item("maturin_extension.othermod", m.getattr("othermod")?)?;
    sys_modules.set_item("maturin_extension.pyclass_iter", m.getattr("pyclass_iter")?)?;
    sys_modules.set_item("maturin_extension.subclassing", m.getattr("subclassing")?)?;

    Ok(())
}
