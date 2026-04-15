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

pub mod listobject {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::listobject::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::listobject::*;
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
