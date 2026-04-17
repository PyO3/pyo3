#[path = "cpython/mod.rs"]
pub mod cpython;
pub mod current;
#[path = "rustpython/mod.rs"]
pub mod rustpython;

macro_rules! backend_pybuffer_item {
    ($item:item) => {
        #[cfg(any(Py_3_11, PyRustPython))]
        $item
    };
}
pub(crate) use backend_pybuffer_item;

macro_rules! backend_rustpython_item {
    ($item:item) => {
        #[cfg(PyRustPython)]
        $item
    };
}
pub(crate) use backend_rustpython_item;

macro_rules! backend_non_rustpython_item {
    ($item:item) => {
        #[cfg(not(PyRustPython))]
        $item
    };
}
pub(crate) use backend_non_rustpython_item;

macro_rules! backend_rustpython_mod {
    ($name:ident, $path:literal) => {
        #[cfg_attr(PyRustPython, path = $path)]
        mod $name;
    };
}
pub(crate) use backend_rustpython_mod;

macro_rules! backend_runtime_support {
    () => {
        #[cfg(PyRustPython)]
        mod rustpython_runtime;

        #[cfg(PyRustPython)]
        pub fn rustpython_runtime_thread_id() -> Option<std::thread::ThreadId> {
            rustpython_runtime::runtime_thread_id()
        }
    };
}
pub(crate) use backend_runtime_support;

macro_rules! backend_cpython_exports {
    () => {
        #[cfg(all(not(Py_LIMITED_API), not(PyRustPython)))]
        mod cpython;

        #[cfg(all(not(Py_LIMITED_API), not(PyRustPython)))]
        pub use self::cpython::*;
    };
}
pub(crate) use backend_cpython_exports;
