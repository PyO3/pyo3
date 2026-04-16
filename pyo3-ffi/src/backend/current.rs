pub mod datetime {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::datetime::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::datetime::*;
}

pub mod pybuffer {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::pybuffer::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::pybuffer::*;
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

pub mod warnings {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::warnings::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::warnings::*;
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
