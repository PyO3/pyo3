use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::wrap_pymodule;

pub mod awaitable;
pub mod buf_and_str;
pub mod comparisons;
pub mod datetime;
pub mod dict_iter;
pub mod enums;
pub mod misc;
pub mod objstore;
pub mod othermod;
pub mod path;
pub mod pyclasses;
pub mod pyfunctions;
pub mod sequence;
pub mod subclassing;

#[pymodule(gil_used = false)]
fn pyo3_pytests(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    PyModule::add_wrapped(m, wrap_pymodule!(awaitable::awaitable))?;
    #[cfg(not(Py_LIMITED_API))]
    PyModule::add_wrapped(m, wrap_pymodule!(buf_and_str::buf_and_str))?;
    PyModule::add_wrapped(m, wrap_pymodule!(comparisons::comparisons))?;
    #[cfg(not(Py_LIMITED_API))]
    PyModule::add_wrapped(m, wrap_pymodule!(datetime::datetime))?;
    PyModule::add_wrapped(m, wrap_pymodule!(dict_iter::dict_iter))?;
    PyModule::add_wrapped(m, wrap_pymodule!(enums::enums))?;
    PyModule::add_wrapped(m, wrap_pymodule!(misc::misc))?;
    PyModule::add_wrapped(m, wrap_pymodule!(objstore::objstore))?;
    PyModule::add_wrapped(m, wrap_pymodule!(othermod::othermod))?;
    PyModule::add_wrapped(m, wrap_pymodule!(path::path))?;
    PyModule::add_wrapped(m, wrap_pymodule!(pyclasses::pyclasses))?;
    PyModule::add_wrapped(m, wrap_pymodule!(pyfunctions::pyfunctions))?;
    PyModule::add_wrapped(m, wrap_pymodule!(sequence::sequence))?;
    PyModule::add_wrapped(m, wrap_pymodule!(subclassing::subclassing))?;

    // Inserting to sys.modules allows importing submodules nicely from Python
    // e.g. import pyo3_pytests.buf_and_str as bas

    let sys = PyModule::import(py, "sys")?;
    let sys_modules = sys.getattr("modules")?.downcast_into::<PyDict>()?;
    sys_modules.set_item("pyo3_pytests.awaitable", m.getattr("awaitable")?)?;
    sys_modules.set_item("pyo3_pytests.buf_and_str", m.getattr("buf_and_str")?)?;
    sys_modules.set_item("pyo3_pytests.comparisons", m.getattr("comparisons")?)?;
    sys_modules.set_item("pyo3_pytests.datetime", m.getattr("datetime")?)?;
    sys_modules.set_item("pyo3_pytests.dict_iter", m.getattr("dict_iter")?)?;
    sys_modules.set_item("pyo3_pytests.enums", m.getattr("enums")?)?;
    sys_modules.set_item("pyo3_pytests.misc", m.getattr("misc")?)?;
    sys_modules.set_item("pyo3_pytests.objstore", m.getattr("objstore")?)?;
    sys_modules.set_item("pyo3_pytests.othermod", m.getattr("othermod")?)?;
    sys_modules.set_item("pyo3_pytests.path", m.getattr("path")?)?;
    sys_modules.set_item("pyo3_pytests.pyclasses", m.getattr("pyclasses")?)?;
    sys_modules.set_item("pyo3_pytests.pyfunctions", m.getattr("pyfunctions")?)?;
    sys_modules.set_item("pyo3_pytests.sequence", m.getattr("sequence")?)?;
    sys_modules.set_item("pyo3_pytests.subclassing", m.getattr("subclassing")?)?;

    Ok(())
}
