pub mod runtime {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::runtime::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::runtime::*;
}

pub mod err_state {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::err_state::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::err_state::*;
}

pub mod pyclass {
    #[cfg(PyRustPython)]
    pub use crate::backend::rustpython::pyclass::*;
    #[cfg(not(PyRustPython))]
    pub use crate::backend::cpython::pyclass::*;
}
