#[cfg(all(not(Py_LIMITED_API), not(PyPy)))]
include!("cpython/initconfig.rs");
