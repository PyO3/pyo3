use pyo3::prelude::*;

pub mod buf_and_str;
pub mod comparisons;
pub mod datetime;
pub mod dict_iter;
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
    use pyo3::types::{PyDict, PyModule};
    use pyo3::PyResult;

    #[pyo3]
    use {
        crate::comparisons::comparisons, crate::dict_iter::dict_iter, crate::misc::misc,
        crate::objstore::objstore, crate::othermod::othermod, crate::path::path,
        crate::pyclasses::pyclasses, crate::pyfunctions::pyfunctions, crate::sequence::sequence,
        crate::subclassing::subclassing,
    };

    #[pyo3]
    #[cfg(not(Py_LIMITED_API))]
    use {crate::buf_and_str::buf_and_str, crate::datetime::datetime};

    #[pymodule_init]
    fn init(m: &PyModule) -> PyResult<()> {
        let sys = PyModule::import(m.py(), "sys")?;
        let sys_modules: &PyDict = sys.getattr("modules")?.downcast()?;
        #[cfg(not(Py_LIMITED_API))]
        sys_modules.set_item("pyo3_pytests.buf_and_str", m.getattr("buf_and_str")?)?;
        sys_modules.set_item("pyo3_pytests.comparisons", m.getattr("comparisons")?)?;
        #[cfg(not(Py_LIMITED_API))]
        sys_modules.set_item("pyo3_pytests.datetime", m.getattr("datetime")?)?;
        sys_modules.set_item("pyo3_pytests.dict_iter", m.getattr("dict_iter")?)?;
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
