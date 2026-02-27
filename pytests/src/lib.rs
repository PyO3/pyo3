use pyo3::prelude::*;
use pyo3::types::PyDict;

mod awaitable;
mod buf_and_str;
mod comparisons;
mod consts;
#[cfg(not(Py_LIMITED_API))]
mod datetime;
mod dict_iter;
mod enums;
mod exception;
mod misc;
mod objstore;
mod othermod;
mod path;
mod pyclasses;
mod pyfunctions;
mod sequence;
mod subclassing;

#[doc = include_str!("../MODULE_DOC.md")]
#[pymodule]
mod pyo3_pytests {
    use super::*;

    #[cfg(any(not(Py_LIMITED_API), Py_3_11))]
    #[pymodule_export]
    use buf_and_str::buf_and_str;

    #[cfg(not(Py_LIMITED_API))]
    #[pymodule_export]
    use datetime::datetime;

    #[pymodule_export]
    use {
        awaitable::awaitable, comparisons::comparisons, consts::consts, dict_iter::dict_iter,
        enums::enums, exception::exception, misc::misc, objstore::objstore, othermod::othermod,
        path::path, pyclasses::pyclasses, pyfunctions::pyfunctions, sequence::sequence,
        subclassing::subclassing,
    };

    // Inserting to sys.modules allows importing submodules nicely from Python
    // e.g. import pyo3_pytests.buf_and_str as bas
    #[pymodule_init]
    fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
        let sys = PyModule::import(m.py(), "sys")?;
        let sys_modules = sys.getattr("modules")?.cast_into::<PyDict>()?;
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
}
