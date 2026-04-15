pub mod datetime {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::datetime::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::datetime::*;
}

pub mod complexobject {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::complexobject::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::complexobject::*;
}

pub mod bytesobject {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::bytesobject::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::bytesobject::*;
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
