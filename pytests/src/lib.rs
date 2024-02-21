use pyo3::prelude::*;

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

#[pymodule]
mod pyo3_pytests {
    use super::*;
    use pyo3::types::PyDict;
    #[pymodule_export]
    use {
        awaitable::awaitable, comparisons::comparisons, dict_iter::dict_iter, enums::enums,
        misc::misc, objstore::objstore, othermod::othermod, path::path, pyclasses::pyclasses,
        pyfunctions::pyfunctions, sequence::sequence, subclassing::subclassing,
    };
    #[cfg(not(Py_LIMITED_API))]
    #[pymodule_export]
    use {buf_and_str::buf_and_str, datetime::datetime};

    #[pymodule_init]
    fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
        // Inserting to sys.modules allows importing submodules nicely from Python
        // e.g. import pyo3_pytests.buf_and_str as bas
        let sys = PyModule::import_bound(m.py(), "sys")?;
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
}
