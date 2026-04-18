#[cfg(not(Py_LIMITED_API))]
pub mod datetime {
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::datetime::*;
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::datetime::*;
}

pub mod critical_section {
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::critical_section::*;
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::critical_section::*;
}

pub mod lock {
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::lock::*;
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::lock::*;
}

pub mod descrobject {
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::descrobject::*;
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::descrobject::*;
}

pub mod pyerrors {
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::pyerrors::*;
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::pyerrors::*;
}

#[cfg(any(Py_3_11, PyRustPython))]
pub mod pybuffer {
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::pybuffer::*;
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::pybuffer::*;
    #[cfg(PyRustPython)]
    pub(crate) use crate::backend::rustpython::pybuffer::{BufferViewState, HeapTypeBufferView};
}

pub mod boolobject {
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::boolobject::*;
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::boolobject::*;
}

pub mod bytearrayobject {
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::bytearrayobject::*;
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::bytearrayobject::*;
}

pub mod complexobject {
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::complexobject::*;
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::complexobject::*;
}

pub mod floatobject {
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::floatobject::*;
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::floatobject::*;
}

pub mod longobject {
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::longobject::*;
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::longobject::*;
}

pub mod moduleobject {
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::moduleobject::*;
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::moduleobject::*;
}

pub mod bytesobject {
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::bytesobject::*;
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::bytesobject::*;
}

pub mod pycapsule {
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::pycapsule::*;
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::pycapsule::*;
}

pub mod pymem {
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::pymem::*;
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::pymem::*;
}

pub mod refcount {
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::refcount::*;
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::refcount::*;
}

pub mod dictobject {
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::dictobject::*;
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::dictobject::*;
}

pub mod listobject {
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::listobject::*;
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::listobject::*;
}

pub mod setobject {
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::setobject::*;
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::setobject::*;
}

pub mod tupleobject {
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::tupleobject::*;
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::tupleobject::*;
}

pub mod sliceobject {
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::sliceobject::*;
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::sliceobject::*;
}

pub mod warnings {
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::warnings::*;
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::warnings::*;
}

pub mod weakrefobject {
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::weakrefobject::*;
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::weakrefobject::*;
}

pub mod unicodeobject {
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::unicodeobject::*;
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::unicodeobject::*;
}

pub mod object {
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::object::*;
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::object::*;
}

pub mod runtime {
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::runtime::*;
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::runtime::*;
}

pub mod compat_py_3_9 {
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::compat_py_3_9::*;
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::compat_py_3_9::*;
}
