pub mod datetime {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::datetime::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::datetime::*;
}

pub mod critical_section {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::critical_section::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::critical_section::*;
}

pub mod descrobject {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::descrobject::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::descrobject::*;
}

pub mod pyerrors {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::pyerrors::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::pyerrors::*;
}

pub mod pybuffer {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::pybuffer::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::pybuffer::*;
    #[cfg(PyRustPython)]
    pub(crate) use crate::backend::rustpython::pybuffer::{BufferViewState, HeapTypeBufferView};
}

pub mod boolobject {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::boolobject::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::boolobject::*;
}

pub mod bytearrayobject {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::bytearrayobject::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::bytearrayobject::*;
}

pub mod complexobject {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::complexobject::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::complexobject::*;
}

pub mod floatobject {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::floatobject::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::floatobject::*;
}

pub mod longobject {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::longobject::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::longobject::*;
}

pub mod moduleobject {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::moduleobject::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::moduleobject::*;
}

pub mod bytesobject {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::bytesobject::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::bytesobject::*;
}

pub mod pycapsule {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::pycapsule::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::pycapsule::*;
}

pub mod pymem {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::pymem::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::pymem::*;
}

pub mod refcount {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::refcount::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::refcount::*;
}

pub mod dictobject {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::dictobject::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::dictobject::*;
}

pub mod listobject {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::listobject::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::listobject::*;
}

pub mod setobject {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::setobject::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::setobject::*;
}

pub mod tupleobject {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::tupleobject::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::tupleobject::*;
}

pub mod sliceobject {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::sliceobject::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::sliceobject::*;
}

pub mod warnings {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::warnings::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::warnings::*;
}

pub mod weakrefobject {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::weakrefobject::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::weakrefobject::*;
}

pub mod unicodeobject {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::unicodeobject::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::unicodeobject::*;
}

pub mod object {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::object::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::object::*;
}

pub mod runtime {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::runtime::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::runtime::*;
}
